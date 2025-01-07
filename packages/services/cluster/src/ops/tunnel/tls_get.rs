use chirp_workflow::prelude::*;

use crate::types::TlsState;

#[derive(Debug)]
pub struct Input {}

#[derive(Debug)]
pub struct Output {
	pub cert_pem: String,
	pub private_key_pem: String,
}

#[operation]
pub async fn cluster_datacenter_tls_get(ctx: &OperationCtx, input: &Input) -> GlobalResult<Output> {
	let row = sql_fetch_optional!(
		[ctx, (String, String)]
		"
		SELECT cert_pem, private_key_pem
		FROM db_cluster.tunnel_tls
		WHERE state != $1
		",
		TlsState::Creating as i64,
	)
	.await?;
	let (cert_pem, private_key_pem) = unwrap!(row, "tunnel tls not created yet");

	Ok(Output {
		cert_pem,
		private_key_pem,
	})
}
