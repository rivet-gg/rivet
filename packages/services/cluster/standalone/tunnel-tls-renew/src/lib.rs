use chirp_workflow::prelude::*;
use cluster::types::TlsState;
use openssl::asn1::Asn1Time;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::extension::{ExtendedKeyUsage, SubjectAlternativeName};
use openssl::x509::{X509NameBuilder, X509Req, X509};

// How much time before the cert expires to renew it
const EXPIRE_PADDING: i64 = util::duration::days(30);
// Days
const VALIDITY_PERIOD: u32 = 365;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config.clone(), pools.clone(), util::timestamp::now()).await
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
	ts: i64,
) -> GlobalResult<()> {
	let client =
		chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("cluster-tunnel-tls-issue");
	let cache = rivet_cache::CacheInner::from_env(pools.clone())?;
	let ctx = StandaloneCtx::new(
		chirp_workflow::compat::db_from_pools(&pools).await?,
		config,
		rivet_connection::Connection::new(client, pools, cache),
		"cluster-tunnel-tls-issue",
	)
	.await?;

	let (create_cert,) = sql_fetch_one!(
		[ctx, (bool,)]
		"
		WITH
			-- NULL if row does not exist, true if within EXPIRE_PADDING and state is Active
			selected AS (
				SELECT (state = $5 AND expire_ts < $1 + $2) AS expiring
				FROM db_cluster.tunnel_tls
			),
			updating AS (
				INSERT INTO db_cluster.tunnel_tls (_id, cert_pem, private_key_pem, state, expire_ts)
				VALUES (0, NULL, NULL, $3, $1)
				ON CONFLICT (_id) DO UPDATE
				SET state = CASE WHEN (SELECT expiring FROM selected)
					THEN $4
					ELSE db_cluster.tunnel_tls.state
				END
				RETURNING 1
			)
		SELECT NOT EXISTS(SELECT 1 FROM selected) OR (SELECT expiring FROM selected)
		",
		ts,
		EXPIRE_PADDING,
		TlsState::Creating as i64,
		TlsState::Renewing as i64,
		TlsState::Active as i64,
	)
	.await?;

	if create_cert {
		tracing::info!("creating new tunnel cert");

		let tls_config = &ctx.config().server()?.tls()?;
		let ca_cert_pem = tls_config.root_ca_cert_pem.read();
		let ca_key_pem = tls_config.root_ca_key_pem.read();
		
		let ca_key = PKey::private_key_from_pem(ca_cert_pem.as_bytes())?;
		let ca_cert = X509::from_pem(ca_key_pem.as_bytes())?;

		let key = rivet_tls::generate_key(2048)?;
		let cert = generate_signed_cert(
			&key,
			"Tunnel Client",
			&ca_key,
			&ca_cert,
			VALIDITY_PERIOD,
			false,
			Some(&["*.tunnel.rivet.gg"]),
		)?;

		let cert_pem = String::from_utf8(cert.to_pem()?)?;
		let private_key_pem = String::from_utf8(key.private_key_to_pem_pkcs8()?)?;
		let expire_ts = util::timestamp::now() + util::duration::days(VALIDITY_PERIOD as i64);

		sql_execute!(
			[ctx]
			"
			UPDATE db_cluster.tunnel_tls
			SET
				cert_pem = $1,
				private_key_pem = $2,
				state = $3,
				expire_ts = $4
			",
			cert_pem,
			private_key_pem,
			TlsState::Active as i64,
			expire_ts,
		)
		.await?;
	}

	Ok(())
}
