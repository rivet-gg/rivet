CREATE TABLE cluster_config (
    cluster_id UUID PRIMARY KEY,
	config BYTES NOT NULL,
);

CREATE TABLE servers (
	server_id UUID PRIMARY KEY,
	datacenter_id UUID NOT NULL,
    cluster_id UUID NOT NULL REFERENCES cluster_config (cluster_id),
	server_type INT NOT NULL,

	-- Null until actual server is provisioned
	provider_server_id TEXT,
	vlan_ip TEXT,
	public_ip TEXT,

	-- Null until nomad node successfully registers
	nomad_node_id TEXT,

	create_ts INT NOT NULL,
	nomad_join_ts INT,
	-- Null if not draining
	drain_ts INT,
	-- When the server was marked to be deleted from the cloud provider
	destroy_ts INT,
);
