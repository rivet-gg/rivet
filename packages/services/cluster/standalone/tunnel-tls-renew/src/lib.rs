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
		let ca_private_key_pem = tls_config.root_ca_key_pem.read();
		let expire_ts = util::timestamp::now() + util::duration::days(VALIDITY_PERIOD as i64);

		// Generate private key
		let rsa_key = Rsa::generate(2048)?;
		let private_key = PKey::from_rsa(rsa_key)?;

		// Create X.509 certificate request
		let mut name_builder = X509NameBuilder::new()?;
		name_builder.append_entry_by_text("CN", "Tunnel Client")?;
		name_builder.append_entry_by_text("O", "Rivet Gaming, Inc.")?;
		let name = name_builder.build();

		let mut req = X509Req::builder()?;
		req.set_subject_name(&name)?;
		req.set_pubkey(&private_key)?;
		req.sign(&private_key, openssl::hash::MessageDigest::sha256())?;
		let cert_req = req.build();

		// Load CA private key and certificate
		let ca_cert = X509::from_pem(ca_cert_pem.as_bytes())?;
		let ca_key = PKey::private_key_from_pem(ca_private_key_pem.as_bytes())?;

		// Create certificate from the certificate request
		let mut cert_builder = X509::builder()?;
		cert_builder.set_version(2)?;
		cert_builder.set_subject_name(cert_req.subject_name())?;
		cert_builder.set_issuer_name(ca_cert.subject_name())?;
		cert_builder.set_pubkey(&private_key)?;
		cert_builder.set_not_before(Asn1Time::days_from_now(0)?.as_ref())?;
		cert_builder.set_not_after(Asn1Time::days_from_now(VALIDITY_PERIOD)?.as_ref())?; // 1 year validity

		// Add extensions
		let key_usage = ExtendedKeyUsage::new().client_auth().build()?;
		cert_builder.append_extension(key_usage)?;

		let san = SubjectAlternativeName::new()
			.dns("*.tunnel.rivet.gg")
			.build(&cert_builder.x509v3_context(Some(&ca_cert), None))?;
		cert_builder.append_extension(san)?;

		// Sign the certificate
		cert_builder.sign(&ca_key, openssl::hash::MessageDigest::sha256())?;
		let cert = cert_builder.build();

		let cert_pem = cert.to_pem()?;
		let cert_pem = std::str::from_utf8(&cert_pem)?;
		let private_key_pem = private_key.private_key_to_pem_pkcs8()?;
		let private_key_pem = std::str::from_utf8(&private_key_pem)?;

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
