ALTER TABLE builds
	ADD allocation_type INT NOT NULL DEFAULT 0,
	ADD allocation_total_slots INT NOT NULL DEFAULT 1;
