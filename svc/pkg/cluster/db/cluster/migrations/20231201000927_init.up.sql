CREATE TABLE clusters (
    cluster_id UUID PRIMARY KEY,
	name_id TEXT NOT NULL,
	owner_team_id UUID,
	create_ts INT NOT NULL
);

CREATE TABLE datacenters (
    datacenter_id UUID PRIMARY KEY,
	cluster_id UUID NOT NULL REFERENCES clusters (cluster_id),
	config BYTES NOT NULL,
	name_id TEXT NOT NULL,

	UNIQUE (cluster_id, name_id),
	INDEX (cluster_id)
);

CREATE TABLE servers (
	server_id UUID PRIMARY KEY,
	datacenter_id UUID NOT NULL,
    cluster_id UUID NOT NULL REFERENCES clusters (cluster_id),
	pool_type INT NOT NULL,

	-- Null until actual server is provisioned
	provider_server_id TEXT,
	vlan_ip TEXT,
	network_idx INT,
	public_ip TEXT,

	-- Null until nomad node successfully registers
	nomad_node_id TEXT,

	create_ts INT NOT NULL,
	nomad_join_ts INT,
	-- Null if not draining
	drain_ts INT,
	-- When the server was marked to be deleted by rivet
	cloud_destroy_ts INT
);

-- Stores data for destroying linode resources
CREATE TABLE linode_misc (
	server_id UUID PRIMARY KEY REFERENCES servers (server_id),
	ssh_key_id INT NOT NULL,
	linode_id INT,
	firewall_id INT
);
