CREATE TABLE servers (
	server_id UUID PRIMARY KEY,
	datacenter_id UUID NOT NULL REFERENCES datacenters (datacenter_id),
	pool_type INT NOT NULL, -- rivet.backend.cluster.PoolType

	-- Null until actual server is provisioned
	provider_server_id TEXT,
	provider_hardware TEXT,
	vlan_ip INET,
	network_idx INT,
	public_ip INET,

	-- Null until nomad node successfully registers
	nomad_node_id TEXT,

	create_ts INT NOT NULL,
	provision_complete_ts INT,
	install_complete_ts INT,
	nomad_join_ts INT,
	-- Null if not draining
	drain_ts INT,
	drain_complete_ts INT,
	-- When the server was marked to be deleted by rivet
	cloud_destroy_ts INT,
	taint_ts INT
);


CREATE TABLE docker_images (
	image_id UUID PRIMARY KEY,
	image_name TEXT NOT NULL,
	image_tag TEXT NOT NULL,
	image_digest TEXT NOT NULL,
	image_size INT NOT NULL,
	image_created_ts INT NOT NULL
);