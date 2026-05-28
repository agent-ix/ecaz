# Review Request: Representative Smoke Query Selection

## Summary

The Graviton representative pass advanced past the previous tunnel restart failure, loaded all three remote shards, materialized remote leaves, and published static remote placements. It then failed in the smoke harness because `smoke-customscan-read.sql` assumed the query table contains `id = 0`.

The representative prepared query file starts at ID `100000`, so the smoke `\gset` returned no rows before it could execute CustomScan/read-profile checks. This is AWS harness query selection, not a SPIRE placement or remote scan failure.

This change updates smoke query selection to use the first available query row:

- `smoke-customscan-read.sql`: `ORDER BY id LIMIT 1`
- `smoke.sh` production-read-profile smoke SQL: `ORDER BY id LIMIT 1`
- representative preflight now requires those smoke scripts to keep the ordered first-query selection

## Evidence

See `artifacts/manifest.md`.

Key failure evidence:

- `artifacts/aws-failure/run-representative-performance-pass.log`: real corpus prepared, remote placements published, then smoke failed with `no rows returned for \gset`
- `artifacts/aws-failure/placement-remotes.json`: nodes 2, 3, and 4 were registered as remote placement targets
- `artifacts/aws-failure/aws-running-after-failure.log`: no pending/running/stopping EC2 after teardown

Key local gate:

- `artifacts/preflight-representative-performance.log`: representative preflight passes with the smoke query selection guard
- `artifacts/aws-running-after-local-gate.log`: no pending/running/stopping EC2 after local gate

## Review Focus

- Confirm `ORDER BY id LIMIT 1` is correct for both synthetic and representative query tables.
- Confirm this remains a harness fix and does not mask remote placement/read failures.
- Confirm the preflight guard is narrow enough to catch this specific representative smoke regression before AWS.
