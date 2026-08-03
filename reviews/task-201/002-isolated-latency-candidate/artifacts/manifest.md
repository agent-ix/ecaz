# Task 201 packet 002: isolated MAT-40 candidate

- Head SHA: `c830b184fe4c750936ab13eab2891f63f06ba3d0`
- Task bucket: `reviews/task-201/002-isolated-latency-candidate/`
- Run artifact root: `artifacts/run-fresh/`
- Suite config: `artifacts/task201-mat40-100k-reuse.json` (the preregistered reuse filename was retained; the final run is the fresh step named `task201-mat40-owner-plan-cache-100k-fresh`)
- Suite config SHA-256: `bdb2e45096a83a152951e22471f5d59e050f5931f153c34da3e2be889978f798`
- Results SHA-256: `44ddb7e524053774896bedf8ac436ea04daee464940925afc4807e95a6b30a6b`
- Suite manifest SHA-256: `5fcc3cbb5bd940928dbb0b83808d74aded38f5e5b73cc5109edc3eee11206b29`
- Extension: PG18 release profile, source SHA `c830b184fe4c750936ab13eab2891f63f06ba3d0`
- CLI: `/home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz`; SHA-256 `8496663553699d4350755b771d98655d6a9adea75e112c50c5ac5524797497e3`
- Corpus: `ec_real_100k`; staged query SHA-256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Timestamp: `2026-08-03 11:43:41 -0700` (results artifact mtime)
- Command: `CARGO_TARGET_DIR=/home/peter/dev/ecaz/.worktrees/task201/target /home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz bench suite run --config reviews/task-201/002-isolated-latency-candidate/artifacts/task201-mat40-100k-reuse.json --artifact-dir reviews/task-201/002-isolated-latency-candidate/artifacts/run-fresh`
- Fixture: fresh shared-table physical 3-owner PG18 fixture under `/home/peter/.ecaz/clusters/task201-mat40-100k`; no single-index control.
- Settings held constant: trained 4096-landmark exact head, persisted-head seed digest, BW4/H100, graph degree 32, RaBitQ neighbors, exact ranking, lazy10 materialization, normal traversal replica, 200 queries, 50 warm latency samples, 10 warmups.
- Isolated toggle: `owner_payload_plan_cache=false` versus `true` (MAT-40). Both arms used the same seed digest and replica topology.

The durable key lines are in `artifacts/run-fresh/mat40-owner-plan-cache-100k/distann-multinode-summary.log`; the structured source is `artifacts/run-fresh/results.jsonl`.

## Result and advancement

At 100k, MAT-40 preserved recall at 0.9625 and storage at 3,188,056,064 physical-generation bytes. Mean latency was 16.50 ms control versus 16.00 ms candidate (3.0% lower); p50 was 16.60 versus 15.80 ms, p95 19.00 versus 18.90 ms, and p99 20.00 versus 19.60 ms. Remote rows (332) and payload bytes (8,350,092) were unchanged. The candidate therefore advances to the required 10k/50k/100k release matrix in packet 003; it is not promoted from this screen alone.

All materialization correctness scenarios passed, including mixed local/remote, null payload, external TOAST, rejected windows, and post-first-batch remote failure.
