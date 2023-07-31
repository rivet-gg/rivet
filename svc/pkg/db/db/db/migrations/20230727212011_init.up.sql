-- Databases
CREATE TABLE databases (
	database_id UUID PRIMARY KEY,
	database_id_short TEXT NOT NULL UNIQUE,
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

-- Game version
CREATE TABLE game_versions (
	version_id UUID PRIMARY KEY,  -- References db-game.versions
	database_name_id TEXT NOT NULL,  -- References databases (owner_team_id, name_id)
	schema BYTES NOT NULL,
	database_id UUID NOT NULL REFERENCES databases (database_id)
);

