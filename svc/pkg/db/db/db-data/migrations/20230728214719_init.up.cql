CREATE TABLE kv (
	database_id uuid,
	collection text,
	key text,
	value blob,
	update_id uuid,
	PRIMARY KEY ((database_id, collection, key))
);

