CREATE TABLE servers (
	server_id UUID PRIMARY KEY,
	game_id UUID NOT NULL,
	datacenter_id UUID NOT NULL,
	-- The server will be locked to a certain cluster, but a game might change
	-- clusters, and therefore the server will be moved to a new cluster.
	cluster_id UUID NOT NULL,
	-- This represents a map<string, json>
	metadata JSONB NOT NULL,
	resources_cpu_millicores INT NOT NULL,
	resources_memory_mib INT NOT NULL,
	kill_timeout_ms INT NOT NULL,
	
	create_ts INT NOT NULL,
	-- When the server was marked to be deleted by Rivet
	destroy_ts INT,

	INDEX (game_id)
);

CREATE TABLE docker_runtimes (
	server_id UUID PRIMARY KEY REFERENCES servers(server_id),
	image_id UUID NOT NULL,
	args STRING[],
	network_mode INT NOT NULL, -- rivet.backend.dynamic_servers.DockerNetworkMode
	-- This is a map<string, string>
	environment JSONB NOT NULL
);

CREATE TABLE docker_ports_protocol_game_guard (  
    server_id UUID NOT NULL REFERENCES docker_runtimes(server_id),
    port_name string NOT NULL,
	port_number INT NOT NULL,
    protocol INT NOT NULL, -- rivet.backend.dynamic_servers.GameGuardProtocol

    PRIMARY KEY (server_id, port_name)
);

CREATE TABLE docker_ports_host (  
	server_id UUID NOT NULL REFERENCES docker_runtimes(server_id),
	port_name string NOT NULL,
	port_number INT NOT NULL,

	PRIMARY KEY (server_id, port_name)
);