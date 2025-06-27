CREATE TABLE IF NOT EXISTS runner_logs (
    namespace LowCardinality(String),
    runner_id UUID,
    stream_type UInt8,  -- pegboard::types::LogsStreamType
    ts DateTime64 (9),
    message String,
    actor_id String MATERIALIZED
        if(
            length(extractAll(message, '\\bactor_([a-zA-Z0-9]{12,})\\b')) > 0,
            extractAll(message, '\\bactor_([a-zA-Z0-9]{12,})\\b')[1],
            ''
        )
) ENGINE = ReplicatedMergeTree ()
PARTITION BY
    toStartOfHour (ts)
ORDER BY (
    namespace,
    runner_id,
    toUnixTimestamp (ts),
    stream_type
)
TTL toDate (ts + toIntervalDay (14))
SETTINGS index_granularity = 8192, ttl_only_drop_parts = 1;
