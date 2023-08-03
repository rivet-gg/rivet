CREATE TABLE kv_index_double_asc (
	database_id uuid,
	collection text,
	"index" text,
	group_by frozen<map<text, text>>,
	rank_0 double,
	key text,
	entry blob,
	PRIMARY KEY ((database_id, collection, "index", group_by), rank_0, key)
) WITH CLUSTERING ORDER BY (rank_0 ASC, key ASC);

