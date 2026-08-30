# Task 230 packet 004 final artifact manifest

- Benchmark head SHA: `8bcccb56c6381527c4d2f3a4f4c9931b66b9235c`
- Task bucket: `reviews/task-230/004-full-scale-decision/`
- Packet: frozen full-scale PROMOTE/STOP decision, seq-03
- Timestamp: 2026-08-29 (America/Los_Angeles)
- Lane: local Intel, PostgreSQL 18, release extension, no debug override
- Fixture / format: staged real 1,536-dimensional `ec_real_10k`,
  `ec_real_50k`, and `ec_real_100k`; production DistANN row-heap control versus
  descriptor V4 / Graph V2 hot/cold candidate; no rerank variant
- Isolation: 20 fresh one-index-per-table fixtures; two counterbalanced primary
  pairs per scale and four fresh matched 100k secondary projection pairs
- Run directories: distinct children of `~/.ecaz/clusters`; all removed after
  capture
- Config: `crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json`
- Config SHA-256:
  `e141ac65a7e18eaf4512509c549ba750e3106a2a045942e0eb6a5ac8fcc5437c`
- Final disposition: **STOP**
- Artifact inventory: `artifact-sha256.txt` records and verifies every other
  retained artifact in this packet; its SHA-256 is
  `6fa13d817da902b639b69d465d781b7195a3ef1bd1cc5f112749269f8fd4e8df`.
  The Cargo receipts and dry-run log entries hash Git's committed
  LF-normalized bytes.

## Canonical result surface

### `run/suite-manifest.json` and `run/results.jsonl`

- Command:
  `/home/peter/.cargo-target/debug/ecaz bench suite run --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json --artifact-dir reviews/task-230/004-full-scale-decision/artifacts/run --manifest-output reviews/task-230/004-full-scale-decision/artifacts/run/suite-manifest.json --results-output reviews/task-230/004-full-scale-decision/artifacts/run/results.jsonl`
- Resumes used the same manifest/results paths after cleanup of startup-only
  port collisions; see `step2-port-collision-retry.log`.
- Final result: 20 succeeded steps, zero failed/skipped/dry-run steps, zero
  missing or stale artifacts. The process exits nonzero only because four
  frozen suite recall thresholds fail.
- Each arm directory retains its review-grade `distann-multinode-summary.log`
  and latency log; primary arms additionally retain recall logs and physical
  prediction JSON.

### `suite-status-final.log` and `suite-audit-final.log`

- Status command:
  `/home/peter/.cargo-target/debug/ecaz bench suite status --manifest reviews/task-230/004-full-scale-decision/artifacts/run/suite-manifest.json --results reviews/task-230/004-full-scale-decision/artifacts/run/results.jsonl`
- Status result: `completed=20 failed=0 skipped=0 dry_run=0
  missing_artifacts=0 stale=0`.
- Audit command:
  `/home/peter/.cargo-target/debug/ecaz bench suite audit --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json --manifest reviews/task-230/004-full-scale-decision/artifacts/run/suite-manifest.json --results reviews/task-230/004-full-scale-decision/artifacts/run/results.jsonl`
- Audit result: `audit passed: 20 steps`.

### `io-attribution.md`

- Derived without remeasurement from the 20 retained per-arm summary logs.
- Records every emitted `physical_benchmark_row_tier_io` row: node, tier, all
  six `pg_statio_all_tables` deltas, and relation hit ratio.
- Records every arm's total accesses, hits, aggregate shared-buffer hit ratio,
  elapsed time, and iteration count.

## Entry receipts

### `cargo-pgrx-install-release-pg18.log`

- Command: `cargo pgrx install --release --pg-config /home/peter/.ecaz/toolchains/pg18-ssl/bin/pg_config --no-default-features --features 'pg18 distann-head-attribution-benchmark'`
- Result: exit 0; release extension installed after Packet 003's test install.

### `cargo-build-cli-release-runner.log`

- Command: `cargo build --release -p ecaz-cli`
- Result: exit 0; suite runner built from benchmark head.

### `cargo-clippy-cli-entry.log`

- Command: `cargo clippy -p ecaz-cli --all-targets`
- Result: exit 0; accepted baseline unchanged at 77 binary warnings and 78 test
  warnings.

### `suite-audit-entry.log`

- Command: `/home/peter/.cargo-target/debug/ecaz bench suite audit --config crates/ecaz-cli/suites/task230-hot-cold-10k-50k-100k.json`
- Result: exit 0 before the real run; frozen 20-step config and staged inputs
  admitted.

### `step2-port-collision-retry.log`

- Compact operational receipt for five startup-only PostgreSQL port
  collisions. Each records the failed step and phase, verified-idle port and
  process checks, fixture cleanup, same-manifest resume, and exclusion of a
  failed decision row.

## Key decision lines

- Four 50k recall thresholds fail: all physical arms produce 0.9540–0.9545
  against the frozen 0.980 floor.
- Prediction SHA parity fails in 50k pair B and both 100k pairs.
- 100k recall deltas fail in both pairs (-0.0025 and -0.0020).
- 100k mean gate passes pair A (12.40 → 9.47 ms) and fails pair B
  (8.67 → 9.47 ms); pair B p95/p99 also fail.
- 50k pair-B replacement p95 fails at 2.734× control.
- Tier-laziness, both storage gates, build/publish, insert, and the other DML
  gates pass.
- Direction predictions: id-only/hot-scalar falsified as a general claim;
  exact-vector falsified; cold-only falsified; mixed supported; select-all
  supported.

Operational console replays, PostgreSQL server logs, memory sampling series,
and regenerable membership snapshots were deliberately excluded under the
repository's review-packet policy. Corpus/query/truth data are not committed;
their prefixes and digests are recorded in the suite manifest/results and
per-arm summaries.
