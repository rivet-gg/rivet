CREATE INDEX ON runs (run_id);
CREATE INDEX ON runs (cleanup_ts) WHERE cleanup_ts IS NULL;
CREATE INDEX ON runs (finish_ts) WHERE finish_ts IS NULL;
CREATE INDEX ON run_proxied_ports (ingress_port, proxy_protocol);
CREATE INDEX ON run_meta_nomad (run_id, dispatched_job_id);
