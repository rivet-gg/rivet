ALTER TABLE datacenters
	ADD COLUMN pools2 JSONB, -- Vec<cluster::types::Pool>
	ADD COLUMN provider JSONB, -- cluster::types::Provider
	ADD COLUMN build_delivery_method JSONB; -- cluster::types::BuildDeliveryMethod

ALTER TABLE servers
	ADD COLUMN pool_type2 JSONB; -- cluster::types::PoolType

CREATE TABLE server_images2 (
	provider JSONB, -- cluster::types::Provider
	install_hash TEXT,
	datacenter_id UUID,
	pool_type INT,

	create_ts INT NOT NULL,
	provider_image_id TEXT,

	PRIMARY KEY (provider, install_hash, datacenter_id, pool_type)
);

ALTER TABLE server_images_linode
	ADD COLUMN pool_type2 JSONB; -- cluster::types::PoolType
