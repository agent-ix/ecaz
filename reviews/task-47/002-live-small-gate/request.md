# Review Request: Task 47 Live Recall And Cost Gates

## Summary

This packet promotes Task 47 from entrypoint scaffolding to live PR gates:

- `make recall-gate` runs against generated small PG18 fixtures with documented per-AM floors.
- `make cost-gate` runs the explain suite and then checks `fixtures/cost-queries/baseline.json`.
- `docs/recall-floors.md` now names the fixture contract, current floors, and explicit baseline-update process.
- `tests/recall_integration.rs` is marked as redundant for AM gating and retained only as an ignored pure-Rust quantizer benchmark.

## Key Results

`make recall-gate` passed on the generated 512-row / 64-query fixture:

| AM | Search setting | Observed recall@10 | Floor |
| --- | --- | ---: | ---: |
| HNSW | `ef_search=128` | `0.8500` | `0.84` |
| IVF | `nprobe=48`, `rerank_width=750` | `0.8500` | `0.84` |
| DiskANN | `list_size=200` | `0.5660` | `0.55` |

`make cost-gate` passed:

| Step | Modeled total cost | Baseline |
| --- | ---: | ---: |
| `ivf-small-cost` | `22.4224` | `22.4224` |
| `hnsw-small-cost` | `527.8` | `527.8` |

## Validation

- `cargo test -p ecaz-cli parses_explain_planner_cost_rows_from_psql_aligned_output`
- `make recall-gate ECAZ_ARGS="--database postgres --host /Users/peter/.pgrx --port 28818" GATE_ARGS="--log-file reviews/task-47/002-live-small-gate/artifacts/make-recall-gate.log"`
- `make cost-gate ECAZ_ARGS="--database postgres --host /Users/peter/.pgrx --port 28818" GATE_ARGS="--log-file reviews/task-47/002-live-small-gate/artifacts/make-cost-gate.log"`
- `python3 scripts/check_cost_baseline.py target/gates/cost-small/results.jsonl fixtures/cost-queries/baseline.json`
- `./target/debug/ecaz bench suite audit --config fixtures/gates/recall-gate-small.json`
- `./target/debug/ecaz bench suite audit --config fixtures/gates/recall-gate-full.json`
- `./target/debug/ecaz bench suite audit --config fixtures/gates/cross-am-gate-small.json`
- `./target/debug/ecaz bench suite audit --config fixtures/gates/cost-gate-small.json`

Artifacts are listed in `artifacts/manifest.md`.
