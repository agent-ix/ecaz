# Artifact Manifest: Task 47 Live Small Gate

- Head SHA: `172a5f342f26`
- Task bucket: `reviews/task-47/002-live-small-gate`
- Timestamp: `2026-05-18T18:39:06Z`
- Lane: Task 47 PR recall and planner-cost gates
- Fixture: generated 512-row corpus and 64-query file, 32 dimensions
- Surface: isolated one-index-per-table prefixes
- Prefixes: `task47_gate_hnsw`, `task47_gate_ivf`, `task47_gate_diskann`

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `install-ecaz-pg-test.log` | `./target/debug/ecaz dev install ecaz-pg-test --pg 18` | Sandbox-blocked install attempt kept for traceability. |
| `install-ecaz-pg-test-escalated.log` | Same install command outside sandbox | Installed the extension for PG18. |
| `generate-corpus.log` | `./target/debug/ecaz corpus generate --output target/gates/fixtures/gate_corpus.tsv --n 512 --dim 32 --seed 47` | Generated small corpus. |
| `generate-queries.log` | `./target/debug/ecaz corpus generate --output target/gates/fixtures/gate_queries.tsv --n 64 --dim 32 --seed 48 --start-id 100000 --kind queries` | Generated small query set. |
| `load-hnsw.log` | `./target/debug/ecaz corpus load ... --prefix task47_gate_hnsw --profile ec_hnsw --m 16` | Built `task47_gate_hnsw_m16_idx`. |
| `load-ivf.log` | `./target/debug/ecaz corpus load ... --prefix task47_gate_ivf --profile ec_ivf --reloption nlists=16 --reloption nprobe=48 --reloption rerank_width=750` | Built `task47_gate_ivf_idx`. |
| `load-diskann.log` | `./target/debug/ecaz corpus load ... --prefix task47_gate_diskann --profile ec_diskann --reloption graph_degree=16 --reloption build_list_size=64 --reloption list_size=200 --reloption alpha=1.2` | Built `task47_gate_diskann_idx`. |
| `make-recall-gate.log` | `make recall-gate ECAZ_ARGS="--database postgres --host /Users/peter/.pgrx --port 28818" GATE_ARGS="--log-file reviews/task-47/002-live-small-gate/artifacts/make-recall-gate.log"` | Passed with HNSW recall `0.8500`, IVF recall `0.8500`, DiskANN recall `0.5660`. |
| `recall-results.jsonl` | Copied from `target/gates/recall-small/results.jsonl` | Packet-local normalized recall rows. |
| `make-cost-gate.log` | `make cost-gate ECAZ_ARGS="--database postgres --host /Users/peter/.pgrx --port 28818" GATE_ARGS="--log-file reviews/task-47/002-live-small-gate/artifacts/make-cost-gate.log"` | Passed suite positivity thresholds and generated planner-cost rows. |
| `cost-results.jsonl` | Copied from `target/gates/cost-small/results.jsonl` | Packet-local normalized planner-cost rows. |
| `cost-baseline-check.log` | `python3 scripts/check_cost_baseline.py target/gates/cost-small/results.jsonl fixtures/cost-queries/baseline.json` | Baseline check passed for HNSW and IVF modeled costs, selectivity, correlation, index pages, and reltuples. |
| `audit-*.log` | `./target/debug/ecaz bench suite audit --config fixtures/gates/*.json` | All four Task 47 suite configs audit clean. |
| `scratch-restart.log` | `./target/debug/ecaz dev scratch restart --pg 18` | Sandbox-blocked restart attempt; the existing PG18 socket was reused instead. |

## Key Rows

- Recall: HNSW `0.8500`, IVF `0.8500`, DiskANN `0.5660`.
- Cost: IVF modeled total cost `22.4224`; HNSW modeled total cost `527.8`.
- Cost baseline: all 12 checked fields matched `fixtures/cost-queries/baseline.json`.
