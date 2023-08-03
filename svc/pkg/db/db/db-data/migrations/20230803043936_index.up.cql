CREATE TABLE kv_index (
	database_id uuid,
	collection text,
	"index" text,
	group_by frozen<map<text, text>>,
	entry_id text,
	entry blob,
	PRIMARY KEY ((database_id, collection, "index", group_by), entry_id)
);

