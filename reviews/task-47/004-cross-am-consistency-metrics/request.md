# Review Request: Cross-AM Consistency Metrics

Task: `plan/tasks/47-recall-and-cost-model-gates.md`

Implementation commit: `55de777f29aedad79ec8edee17daf514e6876459`

## Scope

This slice fills the first cross-AM metric gap from the Task 47 feedback plan:

- `ecaz bench recall` can now write per-query top-k prediction JSON via `--predictions-output`.
- New `ecaz bench cross-am` consumes `label=prediction.json` inputs and reports pairwise `jaccard@k` and `kendall_tau@k`.
- `bench suite` understands a `cross-am` step, includes prediction JSON as recall-produced artifacts, audits cross-step dependencies, parses cross-AM tables into `cross_am` result rows, and can threshold those rows.
- `fixtures/gates/cross-am-gate-small.json` now runs HNSW and DiskANN recall steps, then compares their prediction exports.
- `docs/recall-floors.md` documents the report-first cross-AM floors and artifact requirements.

The Kendall metric is intentionally top-k bounded: it ranks the union of both AM top-k lists and assigns missing entries rank `k + 1`, ignoring ties between two missing entries. That makes rank drift visible even when membership differs.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md` for commands and key lines.

- `cargo test -p ecaz-cli cross_am -- --test-threads=1`: 12 passed.
- `cargo check -p ecaz-cli`: passed.
- `cargo run -p ecaz-cli -- bench suite audit --config fixtures/gates/cross-am-gate-small.json`: audit passed for 3 steps.
- `cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/cross-am-gate-small.json --dry-run`: expands the new cross-AM step after both recall prediction outputs.
- Packet-local `./target/debug/ecaz bench cross-am` smoke produced `hnsw~diskann`, `queries=2`, `k=3`, `jaccard@k=0.7500`, `kendall_tau@k=-0.3333`.

## Non-Goals / Remaining Task 47 Gaps

This packet does not close all of Task 47. Remaining known gaps include exact-KNN cache reuse, real/mixed corpus calibration, confidence intervals, live cross-AM burn-in values, expanded cost-model rows, operator override coverage, and deliberate negative fixtures beyond the new shape/depth validation.
