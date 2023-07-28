CREATE TABLE databases (
	database_id UUID PRIMARY KEY,
	owner_team_id UUID NOT NULL,  -- References db-team.teams
	name_id STRING NOT NULL,
	create_ts INT NOT NULL,
	schema BYTES NOT NULL,  -- rivet.backend.db.Schema
	UNIQUE INDEX (owner_team_id, name_id ASC)
);

CREATE TABLE database_schema_history (
	database_id UUID NOT NULL REFERENCES databases (database_id),
	create_ts INT NOT NULL,
	schema BYTES NOT NULL,
	INDEX (database_id, create_ts ASC)
);

