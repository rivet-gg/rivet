CREATE TABLE kv_index_bigint_asc (
	database_id uuid,
	collection text,
	"index" text,
	group_by frozen<map<list<text>, text>>,
	rank_0 bigint,
	entry_id uuid,
	entry blob,
	PRIMARY KEY ((database_id, collection, "index", group_by), rank_0, entry_id)
) WITH CLUSTERING ORDER BY (rank_0 ASC, entry_id ASC);

