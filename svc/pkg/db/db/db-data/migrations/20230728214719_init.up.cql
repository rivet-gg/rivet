CREATE TABLE kv (
	database_id uuid,
	collection text,
	entry_id uuid,
	value blob,
	PRIMARY KEY ((database_id, collection, entry_id))
);

