# Troubleshooting

## Where are pegboard-manager logs?

```bash
cat /var/lib/rivet-client/logs
```

## Where are rivet-isolate-v8-runner logs?

```bash
cat /var/lib/rivet-client/runner/logs
```

## Why don't my runner logs exist?

If there are no logs at `/var/lib/rivet-client/runner/logs`, the runner binary likely failed to spawn.

Common causes:

- The path to the binary is incorrect
- Error loading libraries
- The binary is not set as executable
- The binary is for the wrong architecture

Trying to manually find and run the binary usually resolves these issues.

