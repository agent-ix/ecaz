# Task 204 arm-fidelity packet

- Head SHA for the implementation checkpoint: `615fd72b2d6d31d7bec9020eabcfa8fa34d39a68`.
- Task bucket: `reviews/task-204/001-arm-fidelity/`.
- Fixture/lane: PG18 `distann-local-multinode`, two physical arms at `ec_real_100k`; owner-control versus coordinator-replica.
- Storage format: rabitq neighbor codes; rerank mode is the default co-located exact-distance path.
- Suite config: `artifacts/task204-two-arm-100k-suite.json`.
- Intended command: `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-204/001-arm-fidelity/artifacts/task204-two-arm-100k-suite.json --artifact-dir reviews/task-204/001-arm-fidelity/artifacts/run`.
- Run directory: `/home/peter/.ecaz/clusters/task204-arm-fidelity-100k` (outside the repository and `target/`).
- Isolation: the suite uses one physical three-node fixture with two seed arms; storage rows are emitted inside the arm loop. The fixture is not shared with another packet.
- Timestamp of this packet/preflight: 2026-07-29 America/Los_Angeles.

## Evidence status

The code checkpoint emits `physical_benchmark_storage`,
`physical_benchmark_storage_ratio`, `physical_benchmark_storage_node`,
`physical_benchmark_storage_relation`, and per-arm
`physical_benchmark_traversal_replica_cache` rows. The focused PG18 test passed;
see `pg18-focused.log`.

The required 100k two-arm fixture run is not claimed here: audit and dry-run
passed, but the approved execution fell back to `rows=2000 dim=16` because the
staged corpus files are absent on this host. The run was stopped and its
external cluster removed. See `benchmark-preflight.log`. No benchmark values
are fabricated.

The corrected reread of the committed Task 198/199 artifacts is in
`corrected-198-199-reread.md`.
