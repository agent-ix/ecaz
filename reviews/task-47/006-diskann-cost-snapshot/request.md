# Review Request: DiskANN Cost Snapshot Gate Row

Task: `plan/tasks/47-recall-and-cost-model-gates.md`

Implementation commit: `f6ceced1172d099b80253ace36318b1fbcca4d12`

## Scope

This slice closes the small-fixture DiskANN cost-gate row from the Task 47 reviewer plan:

- adds `ec_diskann_index_cost_snapshot(index_oid oid)` as a stable diagnostic wrapper over the DiskANN planner cost model;
- exposes DiskANN relation options, resolved session/list-size tuning, planner cost constants, modeled cost/selectivity/correlation, and the single-layer DiskANN tree-height assumption;
- wires the bench suite cost snapshot resolver for `ec_diskann`;
- adds `diskann-small-cost` to `fixtures/gates/cost-gate-small.json`;
- updates `fixtures/cost-queries/baseline.json` so the cost baseline now enforces IVF, HNSW, and DiskANN small rows.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `cargo check -p ecaz-cli`: passed.
- `cargo test -p ecaz-cli explain_sql_uses_diskann_profile_guc_and_cost_snapshot -- --test-threads=1`: 1 passed.
- `cargo test -p ecaz-cli parses_explain_planner_cost_rows -- --test-threads=1`: 2 passed.
- `cargo run -p ecaz-cli -- bench suite audit --config fixtures/gates/cost-gate-small.json`: passed, 3 steps.
- `make cost-gate ECAZ_ARGS="--database postgres --host /Users/peter/.pgrx --port 28818" GATE_ARGS="--log-file reviews/task-47/006-diskann-cost-snapshot/artifacts/make-cost-gate.log"`: passed.
- `python3 scripts/check_cost_baseline.py target/gates/cost-small/results.jsonl fixtures/cost-queries/baseline.json`: passed with `diskann-small-cost` enforced.
- `git diff --check`: clean.

The live run produced `diskann-small-cost` with `modeled_startup_cost=816.06`, `modeled_total_cost=816.06`, `index_pages=10`, and `reltuples=512`.

## Notes

The PG18 validation database already had extension version `0.1.1` installed before this slice. After installing the rebuilt extension files, the new SQL symbol was not present in that existing database because the dev extension version did not change. For validation only, `artifacts/register-diskann-cost-snapshot.sql` registered the exact generated C function signature from the installed extension library before rerunning the live gate.

## Remaining Task 47 Gaps

This packet does not close all of Task 47. Remaining gaps include committed exact-KNN fixture caches/regeneration, mixed and real gate corpora, calibrated CI-aware floors, real-corpus/per-node cost rows, `--accept-drift` audit coverage, deliberate negative fixtures, and cross-arch variance evidence.
