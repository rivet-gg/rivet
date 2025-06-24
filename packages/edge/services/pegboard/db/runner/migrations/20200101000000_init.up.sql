CREATE TABLE IF NOT EXISTS actor_runners (
    actor_id String,
    generation UInt32,
    runner_id UUID,
    started_at DateTime64 (9),
    finished_at DateTime64 (9)
) ENGINE = ReplicatedReplacingMergeTree ()
PARTITION BY
    toStartOfHour (started_at)
ORDER BY (
    actor_id,
    generation,
    runner_id
)
TTL toDate (started_at + toIntervalDay (30))
SETTINGS index_granularity = 8192, ttl_only_drop_parts = 1;
