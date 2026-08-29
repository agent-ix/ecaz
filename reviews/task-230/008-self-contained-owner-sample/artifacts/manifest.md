# Task 230 packet 008 artifact manifest

- Head SHA: `177aae194930e0d2958cca02d197daef55277958`
- Task bucket: `reviews/task-230/008-self-contained-owner-sample/`
- Timestamp: 2026-08-29T08:57:48-07:00
- Lane / fixture / storage format / rerank mode: local Intel PG18; real 10k
  one-step hot/cold validation smoke; descriptor V4 / Graph V2; no rerank
- Results state: one validation smoke step succeeded. The run is explicitly not
  Packet 004 decision evidence.

## Failure receipt

`failed-rowheap-owner-sample-summary.log` records the accepted-head release
preflight and pre-measurement malformed sample. No decision row is admitted.

## Static validation

- `cargo-fmt-check.log`: exit 0.
- `cargo-test-focused.log`: exit 0; 1 passed, 0 failed, 552 filtered out.
- `cargo-clippy-cli.log`: exit 0; 77 binary / 78 test warnings.

## Smoke preregistration

- Config: `tiny-hotcold-smoke.json`.
- SHA-256: `7abf830f498f96ac0211714235635cfd760b10d36e01bcddf1aa7ec97394b9ee`.
- Audit: `smoke-audit.log`, exit 0, one step.
- Dry run: `smoke-dry-run.log` and `smoke-dry-run-manifest.json`, one selected
  step; manifest SHA-256
  `e8b8295a0bc7f48f7a0e71edffc9fe4dacfbe07348aa2feee2de11c4cdf2409f`.
- Dry-run runner head: `cb6666410ea75faaf333ca8d017a1d6f044dbbe4`.
- Run directory: `/home/peter/.ecaz/clusters/task230-packet008-hotcold-smoke`;
  removed after durable capture.
- Isolation: fresh one-index-per-table hot/cold fixture; no reuse and no debug
  override.

## Accepted-head install and build

- `cargo-pgrx-install-release-pg18.log`: release extension reinstall at
  `177aae194930e0d2958cca02d197daef55277958`, exit 0.
- `cargo-build-cli.log`: matching `ecaz-cli` build at the same head, exit 0.

## Smoke execution

Command:

```text
/home/peter/.cargo-target/debug/ecaz bench suite run --config reviews/task-230/008-self-contained-owner-sample/artifacts/tiny-hotcold-smoke.json --manifest-output reviews/task-230/008-self-contained-owner-sample/artifacts/smoke-run/suite-manifest.json --results-output reviews/task-230/008-self-contained-owner-sample/artifacts/smoke-run/results.jsonl
```

- `smoke-console.log`: complete suite console; exit 0.
- `smoke-run/suite-manifest.json`: real (not dry-run) manifest; runner head
  `177aae194930e0d2958cca02d197daef55277958`; source config SHA-256
  `7abf830f498f96ac0211714235635cfd760b10d36e01bcddf1aa7ec97394b9ee`;
  one selected step, status `succeeded`, exit code 0.
- `smoke-run/results.jsonl`: normalized output, 172 rows.
- `smoke-run/hotcold/distann-local-multinode.log` and
  `distann-multinode-summary.log`: raw and compact fixture evidence.
- `smoke-run/hotcold/physical-production-latency.log`: raw latency child output;
  exactly 62 `[distann-materialization-work]` rows.
- `smoke-status.log`: `completed=1 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`.
- `smoke-audit-result.log`: audit passed, one step.
- `smoke-key-lines.log`: compact release, remote-owner, row-tier I/O, and
  topology-gate receipt lines.
- `smoke-sha256.log`: config, manifest, and normalized-results hashes.

## Key results

- Release preflight passed on all three nodes, unanimous at accepted head,
  release profile, `debug_override=false`.
- Both remote-owner proofs passed with returned pinned samples and exact-owner
  identities matching expected and materialized source IDs.
- Attribution-work row count: 62, satisfying the Packet 005 contract.
- Id-only row-tier I/O: `pass=true`, cold accesses 0, hot accesses/hits 66/66,
  shared-buffer hit ratio 1.0.
- Topology gate: `pass=true owners=3 remote_verified=2 source_rows=10000`.
- Run directory `/home/peter/.ecaz/clusters/task230-packet008-hotcold-smoke` was
  removed after durable capture.
