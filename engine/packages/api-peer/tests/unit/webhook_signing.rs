use super::*;
static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
#[tokio::test]
async fn raw_body_is_preserved_and_content_encoding_is_rejected() {
	let _guard = TEST_LOCK.lock().await;
	let body = vec![0, 255, b'\r', b'\n', b'{', b'}'];
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from(body.clone()))
		.unwrap();
	let (actual, permit) = read_body(request).await.unwrap();
	assert_eq!(actual, body);
	drop(permit);
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.header("content-encoding", "gzip")
		.body(axum::body::Body::empty())
		.unwrap();
	assert!(read_body(request).await.is_err());
}

#[tokio::test]
async fn unknown_length_upload_charges_before_acceptance_and_releases_capacity_on_failure() {
	let _guard = TEST_LOCK.lock().await;
	use std::sync::atomic::{AtomicU64, Ordering};
	let charged = AtomicU64::new(0);
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from_stream(futures_util::stream::iter([
			Ok::<_, std::io::Error>(vec![0u8; 512 * 1024]),
			Ok(vec![255; 512 * 1024 + 1]),
		])))
		.unwrap();
	let (bytes, permit) = read_body_charged(request, 2 * 1024 * 1024, |n| {
		charged.fetch_add(n, Ordering::Relaxed);
		async { Ok(()) }
	})
	.await
	.unwrap();
	assert_eq!(bytes.len(), 1024 * 1024 + 1);
	assert_eq!(charged.load(Ordering::Relaxed), 2 * 1024 * 1024);
	drop(permit);
	let before = BUFFER_BUDGET.available_permits();
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from(vec![0u8; 1025]))
		.unwrap();
	assert!(
		read_body_charged(request, 1024, |_| async { Ok(()) })
			.await
			.is_err()
	);
	assert_eq!(BUFFER_BUDGET.available_permits(), before);
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from("a"))
		.unwrap();
	assert!(
		read_body_charged(request, 1024, |_| async { Err(Signing::Throttled.build()) })
			.await
			.is_err()
	);
	assert_eq!(BUFFER_BUDGET.available_permits(), before);
}

#[tokio::test]
async fn cancelling_an_upload_releases_its_memory_reservation() {
	let _guard = TEST_LOCK.lock().await;
	let started = std::sync::Arc::new(tokio::sync::Notify::new());
	let observed = started.clone();
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from_stream(futures_util::stream::once(
			async move {
				observed.notify_one();
				std::future::pending::<Result<Vec<u8>, std::io::Error>>().await
			},
		)))
		.unwrap();
	let before = BUFFER_BUDGET.available_permits();
	let task = tokio::spawn(read_body_charged(request, 1024 * 1024, |_| async {
		Ok(())
	}));
	started.notified().await;
	assert!(BUFFER_BUDGET.available_permits() < before);
	task.abort();
	assert!(task.await.unwrap_err().is_cancelled());
	assert_eq!(BUFFER_BUDGET.available_permits(), before);
}

#[tokio::test]
async fn cancelling_a_signer_keeps_memory_reserved_until_blocking_work_stops() {
	let _guard = TEST_LOCK.lock().await;
	let before = BUFFER_BUDGET.available_permits();
	let permit = BUFFER_BUDGET.clone().try_acquire_many_owned(4).unwrap();
	let (started_tx, started_rx) = tokio::sync::oneshot::channel();
	let (release_tx, release_rx) = std::sync::mpsc::channel();
	let task = tokio::spawn(with_signing_permit(permit, move || {
		started_tx.send(()).unwrap();
		release_rx.recv().unwrap();
		Ok(())
	}));
	started_rx.await.unwrap();
	task.abort();
	assert!(task.await.unwrap_err().is_cancelled());
	assert_eq!(BUFFER_BUDGET.available_permits(), before - 4);
	release_tx.send(()).unwrap();
	tokio::time::timeout(std::time::Duration::from_secs(2), async {
		while BUFFER_BUDGET.available_permits() != before {
			tokio::task::yield_now().await;
		}
	})
	.await
	.unwrap();
}

#[tokio::test]
async fn audit_keeps_accepted_bytes_when_a_later_quota_reservation_fails() {
	let _guard = TEST_LOCK.lock().await;
	use std::sync::atomic::{AtomicU64, Ordering};
	let observed = AtomicU64::new(0);
	let calls = AtomicU64::new(0);
	let request = Request::builder()
		.header("content-type", "application/octet-stream")
		.body(axum::body::Body::from_stream(futures_util::stream::iter([
			Ok::<_, std::io::Error>(vec![0; 512 * 1024]),
			Ok(vec![255; 1024 * 1024]),
		])))
		.unwrap();
	let before = BUFFER_BUDGET.available_permits();
	let result = read_body_observed(
		request,
		2 * 1024 * 1024,
		|_| {
			let allowed = calls.fetch_add(1, Ordering::Relaxed) == 0;
			async move {
				if allowed {
					Ok(())
				} else {
					Err(Signing::Throttled.build())
				}
			}
		},
		|bytes| {
			observed.store(bytes, Ordering::Relaxed);
		},
	)
	.await;
	assert!(result.is_err());
	assert_eq!(observed.load(Ordering::Relaxed), 512 * 1024);
	assert_eq!(BUFFER_BUDGET.available_permits(), before);
}
