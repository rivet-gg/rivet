ALTER TABLE datacenters
	ADD COLUMN label BYTES AS (substring(datacenter_id::BYTES FROM 1 FOR 2)) STORED;

CREATE UNIQUE INDEX datacenter_label_idx
ON datacenters (label);
