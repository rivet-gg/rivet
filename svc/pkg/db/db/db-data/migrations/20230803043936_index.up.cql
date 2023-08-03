CREATE TABLE kv_index (
	database_id uuid,
	collection text,
	"index" text,
	group_by frozen<map<list<text>, text>>,
	entry_id uuid,
	entry blob,
	PRIMARY KEY ((database_id, collection, "index", group_by), entry_id)
);

