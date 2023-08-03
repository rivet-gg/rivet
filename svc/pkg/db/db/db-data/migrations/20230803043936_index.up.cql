CREATE TABLE kv_index (
	database_id uuid,
	collection text,
	"index" text,
	group_by frozen<map<text, text>>,
	key text,
	entry blob,
	PRIMARY KEY ((database_id, collection, "index", group_by), key)
);

