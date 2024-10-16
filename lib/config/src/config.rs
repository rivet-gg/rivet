use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Root {
	pub rivet: RivetRoot,
	pub server: Option<Server>,
	pub tokio: Tokio,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RivetRoot {
	pub namespace: String,
	pub cluster_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Server {
	// TODO: Move to secrets
	pub jwt_key: JwtKey,

	pub rivet: Rivet,
	pub cloudflare: Cloudflare,
	pub tls: Tls,
	pub prometheus: Option<Prometheus>,
	pub hcaptcha: Hcaptcha,
	pub turnstile: Turnstile,
	pub sendgrid: Option<Sendgrid>,
	pub chirp: Chirp,
	pub metrics: Metrics,
	pub health: Health,
	pub debug: Debug,

	pub cockroachdb: CockroachDB,
	pub nats: Nats,
	pub clickhouse: Option<ClickHouse>,
	pub redis: RedisTypes,
	pub s3: S3,
	pub nomad: Nomad,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Rivet {
	pub cluster_id: Option<String>,
	pub namespace: Option<String>,

	// TODO: Auto-generate?
	pub source_hash: String,
	pub project_source_hash: String,

	// Networking
	pub ports: Ports,
	pub domain: Domain,
	pub host: Host,
	pub origin: Origin,
	pub api_hub_origin_regex: String,

	// Cluster
	pub default_cluster_config: Option<serde_json::Value>,
	pub job_server_provision_margin: u32,
	pub pb_server_provision_margin: u32,

	// Provision (DNS)
	pub dns_provider: Option<DnsProvider>,

	// Pegboard
	pub pegboard: Pegboard,

	// Accounts
	pub access_kind: RivetAccessKind,
	pub access_token_login: bool,

	// EE
	pub billing: Option<String>,

	// Debug
	pub api_error_verbose: bool,

	// NSFW
	pub profanity_filter_disable: bool,
	pub upload_nsfw_check_enabled: bool,
	pub upload_nsfw_error_verbose: bool,

	// Misc
	pub telemetry_disable: bool,
	pub job_runner_binary_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DnsProvider {
	Cloudflare,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RivetAccessKind {
	Public,
	Private,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Pegboard {
	pub manager_binary_key: String,
	pub container_runner_binary_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct JwtKey {
	pub public: String,
	/// The private EdDSA key in a PEM format. Corresponds to
	/// `rivet_claims::Config::jwt_key_public`.
	pub private: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Ports {
	pub pegboard_ws: u16,
	pub api: u16,
	pub api_internal: u16,
	pub health: u16,
	pub metrics: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Domain {
	pub main: Option<String>,
	pub cdn: Option<String>,
	pub job: Option<String>,
	pub main_api: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Host {
	pub api: Option<String>,
	pub tunnel: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Origin {
	pub api: Option<String>,
	pub hub: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Tokio {
	pub console_enable: bool,
	pub thread_stack_size: Option<usize>,
	pub worker_threads: Option<usize>,
	pub max_blocking_threads: Option<usize>,
	pub global_queue_interval: Option<u64>,
	pub event_interval: Option<u64>,
	pub thread_keep_alive: Option<u64>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Nomad {
	pub url: String,
	pub server_count: usize,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct S3 {
	pub region: String,
	pub endpoint: S3Endpoint,

	// TODO: Move to secrets
	pub access_key_id: String,
	// TODO: Move to secrets
	pub secret_access_key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct S3Endpoint {
	pub internal: String,
	pub external: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RedisTypes {
	pub ephemeral: Redis,
	pub persistent: Redis,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Redis {
	pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Cloudflare {
	pub account_id: String,
	pub zone: CloudflareZone,
	pub backend_dispatcher_namespace: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CloudflareZone {
	pub main: Option<String>,
	pub game: Option<String>,
	pub job: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Tls {
	// TODO: Move to secrets
	pub root_ca_cert_pem: String,
	pub cert_locally_signed_job_cert_pem: String,
	pub cert_locally_signed_job_key_pem: String,
	pub acme: TlsAcme,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct TlsAcme {
	pub directory: TlsAcmeDirectory,
	pub account_private_key_pem: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TlsAcmeDirectory {
	LetsEncrypt,
	LetsEncryptStaging,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Prometheus {
	pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Hcaptcha {
	pub site_key_fallback: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Turnstile {
	pub site_key_main: Option<String>,
	pub site_key_cdn: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Sendgrid {
	pub key: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Chirp {
	pub service_name: String,
	pub worker: ChirpWorker,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ChirpWorker {
	pub kind: String,
	pub instance: String,
	pub rpc_group: Option<String>,
	pub consumer_group: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CockroachDB {
	pub url: String,
	#[serde(default = "CockroachDB::default_min_connections")]
	pub min_connections: u32,
	#[serde(default = "CockroachDB::default_max_connections")]
	pub max_connections: u32,
}

impl CockroachDB {
	fn default_min_connections() -> u32 {
		1
	}

	fn default_max_connections() -> u32 {
		4096
	}
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Nats {
	pub urls: Vec<String>,
	pub username: Option<String>,
	pub password: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct ClickHouse {
	pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Metrics {
	pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Health {
	pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Debug {
	pub disable_rate_limit: bool,
}
