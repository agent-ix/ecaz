# Review Request: Task 123 Production-Read Timeline Instrumentation

## Scope

This packet follows up the reopened Task 121/123 multi-instance feedback and
the instrumentation gap recorded in
`reviews/task-123/009-multi-instance-phase-a-baseline/`.

Code checkpoint `a91955a274fe4ec987f4067ed096f3700131d331` adds per-worker
production-read timeline payload counters and renders them from
`ecaz bench spire-pipeline`.

## What Changed

- `ec_spire_remote_search_production_read_timeline(...)` now returns:
  - `payload_decode_elapsed_ms`
  - `payload_decode_row_count`
  - `payload_decode_bytes`
- `ecaz bench spire-pipeline --include-production-read-profile` now renders a
  `Production read per-node timeline` table.
- The timeline uses the query projection for heap receive payload accounting.
  With the current local multinode suite projection (`id`), heap receive rows
  report projected id payload bytes per worker.

No routing, recall, candidate selection, or storage behavior changed.

## Validation

Focused unit tests:

| Test | Result |
| --- | --- |
| `cargo test -p ecaz-cli spire_pipeline_renders_production_read -- --nocapture` | 2 passed |
| `cargo test -p ecaz-cli spire_pipeline_sql_uses_public_snapshot_contracts -- --nocapture` | 1 passed |
| `cargo test -p ecaz --lib production_read_profile_row_preserves_metric_rollup -- --nocapture` | 1 passed |

Contained multi-instance smoke:

```text
target/debug/ecaz dev spire-multicluster local-multinode-pg18 --tier correctness --artifact-dir reviews/task-123/010-production-read-timeline-instrumentation/artifacts/correctness-smoke --run-dir reviews/task-123/010-production-read-timeline-instrumentation/artifacts/correctness-smoke/run-dir --run-id t123-p10-smoke --coord-port 29918 --remote1-port 29919 --remote2-port 29920 --remote3-port 29921 --bench-top-k 10 --bench-queries-limit 4 --bench-sweep 8 --bench-rowcap-sweep 8 --skip-fault-drills --log-file reviews/task-123/010-production-read-timeline-instrumentation/artifacts/correctness-smoke/local-multinode-command.log
```

Result:

- `SPIRE local multinode fixture passed`
- `HARNESS PASSED`
- production read used `result_source=remote_heap_candidates`
- default production profile: selected pids 32, remote pids 32, dispatches 12,
  remote heap candidates 120
- new per-node timeline default heap rows:
  - node 2: 40 payload rows / 320 bytes, heap p50/p95 25/26 ms
  - node 3: 40 payload rows / 320 bytes, heap p50/p95 24/24 ms
  - node 4: 40 payload rows / 320 bytes, heap p50/p95 26/27 ms

## Relationship To 100k Baseline

Packet `reviews/task-123/009-multi-instance-phase-a-baseline/` remains the 100k
contained multi-instance baseline for `n128 b4/tr50/f8` and `n1024 b2/tr50/f8`.
I did not rerun that full 100k matrix in this packet because the current host
has about 20 GiB free and packet 009 already recorded an ENOSPC failure during
its first 100k attempt before the successful focused rerun.

This packet closes the instrumentation gap by proving per-worker payload-byte
attribution on the same contained multi-instance executor path.

## Evidence

See `artifacts/manifest.md` and `artifacts/extracted-results.md`.

Primary artifacts:

- `artifacts/correctness-smoke/bench-suite/results.jsonl`
- `artifacts/correctness-smoke/bench-suite/production-read-k10-default.log`
- `artifacts/correctness-smoke/bench-suite/production-read-k10-rowcap25k.log`
- `artifacts/correctness-smoke/local-multinode-command.log`
- `artifacts/unit-tests/ecaz-cli-production-read-renderers.log`
- `artifacts/unit-tests/ecaz-cli-sql-contracts.log`
- `artifacts/unit-tests/ecaz-production-profile-rollup.log`

Generated TSVs and packet-local PostgreSQL runtime directories were pruned
before this review request.
