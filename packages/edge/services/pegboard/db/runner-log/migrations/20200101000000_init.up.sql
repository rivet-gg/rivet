
CREATE TABLE IF NOT EXISTS runner_logs (
    runner_id UUID,
    actor_id UUID,      -- When not set will be the NIL UUID (all zeros)
    stream_type UInt8,  -- pegboard::types::LogsStreamType
    ts DateTime64 (9),
    message String
) ENGINE = ReplicatedMergeTree ()
PARTITION BY
    toStartOfHour (ts)
ORDER BY (
    runner_id,
    toUnixTimestamp (ts),
    stream_type
)
TTL toDate (ts + toIntervalDay (3))
SETTINGS index_granularity = 8192, ttl_only_drop_parts = 1;
