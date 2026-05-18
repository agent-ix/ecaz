# Task 47 Packet 004 Artifact Manifest

- head SHA: `55de777f29aedad79ec8edee17daf514e6876459`
- task bucket: `reviews/task-47`
- packet path: `reviews/task-47/004-cross-am-consistency-metrics`
- timestamp: `2026-05-18T22:24:50Z`
- lane: Task 47 cross-AM consistency metrics, small gate wiring
- fixture/storage/rerank mode: packet-local synthetic prediction JSON smoke; `fixtures/gates/cross-am-gate-small.json`; no live PG storage surface in this packet
- isolated one-index-per-table or shared-table surface: not applicable for the packet-local smoke; dry-run gate remains one suite over existing `task47_gate_hnsw` and `task47_gate_diskann` prefixes

## Artifacts

| Artifact | Command | Key lines |
| --- | --- | --- |
| `cargo-test-cross-am.log` | `script -q reviews/task-47/004-cross-am-consistency-metrics/artifacts/cargo-test-cross-am.log cargo test -p ecaz-cli cross_am -- --test-threads=1` | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 340 filtered out` |
| `cargo-check-ecaz-cli.log` | `script -q reviews/task-47/004-cross-am-consistency-metrics/artifacts/cargo-check-ecaz-cli.log cargo check -p ecaz-cli` | `Finished dev profile` |
| `suite-audit-cross-am-small.log` | `script -q reviews/task-47/004-cross-am-consistency-metrics/artifacts/suite-audit-cross-am-small.log cargo run -p ecaz-cli -- bench suite audit --config fixtures/gates/cross-am-gate-small.json` | `[suite:task47-cross-am-gate-small] audit passed: 3 steps` |
| `suite-dry-run-cross-am-small.log` | `script -q reviews/task-47/004-cross-am-consistency-metrics/artifacts/suite-dry-run-cross-am-small.log cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/cross-am-gate-small.json --dry-run` | `bench cross-am --input hnsw=target/gates/cross-am-small/hnsw-real10k-k10-predictions.json --input diskann=target/gates/cross-am-small/diskann-real10k-k10-predictions.json --k 10 --log-output target/gates/cross-am-small/hnsw-diskann-consistency.log` |
| `hnsw-predictions.json` | Static packet-local smoke input matching the `ecaz bench recall --predictions-output` schema. | `profile=ec_hnsw`, `k=3`, `query_ids=[101,102]` |
| `diskann-predictions.json` | Static packet-local smoke input matching the `ecaz bench recall --predictions-output` schema. | `profile=ec_diskann`, `k=3`, `query_ids=[101,102]` |
| `cross-am-smoke-command.log` | `script -q reviews/task-47/004-cross-am-consistency-metrics/artifacts/cross-am-smoke-command.log ./target/debug/ecaz bench cross-am --input hnsw=reviews/task-47/004-cross-am-consistency-metrics/artifacts/hnsw-predictions.json --input diskann=reviews/task-47/004-cross-am-consistency-metrics/artifacts/diskann-predictions.json --k 3 --log-output reviews/task-47/004-cross-am-consistency-metrics/artifacts/cross-am-smoke.log` | `hnsw~diskann`, `queries=2`, `k=3`, `jaccard@k=0.7500`, `kendall_tau@k=-0.3333` |
| `cross-am-smoke.log` | Produced by the smoke command `--log-output`. | Same table row: `hnsw~diskann`, `0.7500`, `-0.3333` |

## Notes

- The `cargo` commands emit pre-existing warnings from `ecaz` library imports and PostgreSQL headers; no warning originates from the new `ecaz-cli` cross-AM module.
- This packet validates the new command, suite expansion, audit dependency graph, and threshold parsing. It does not claim Task 47 completion; live real-corpus calibration and confidence-interval work remain separate gaps.
