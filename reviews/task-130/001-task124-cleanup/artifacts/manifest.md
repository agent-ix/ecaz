# Task 130 Packet 001 Artifact Manifest

- cleanup code SHA: `f61366126` (`Revert "Add reduced-dimension TQ2 rerank format"`)
- task bucket: `reviews/task-130/001-task124-cleanup/`
- packet topic: post-Task-124 cleanup and artifact hygiene
- timestamp: 2026-06-30
- branch: `task-130-tq-cleanup`

## Commands

- `git revert --no-edit 0b3fd57f713297b9c07ceadc364ad6a698021a75`
- `rg "TurboQuant2Dim768|turboquant2_768|tq2_768" src/am/ec_ivf`
- `git check-ignore -v reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/truth-100k-k10.json`
- `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant -- --nocapture`
- `cargo test -p ecaz --lib --no-default-features --features pg18 rerank_format_parse_accepts_turboquant2 -- --nocapture --test-threads=1`
- `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant2_sidecar_uses_compact_qjl_payload -- --nocapture --test-threads=1`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
- `git diff --check`

## Results

- Source search: no `TurboQuant2Dim768`, `turboquant2_768`, or `tq2_768` references remain under `src/am/ec_ivf`.
- Ignore check: `.gitignore:60:reviews/**/truth-*.json` matches packet-local suite truth JSON files.
- Broad `cargo test ... turboquant` attempt: failed. The filter matched unrelated shared counter tests that ran in parallel and poisoned a shared counter mutex after one assertion. This broad filter is not used as the cleanup validation result.
- Focused rerun: `rerank_format_parse_accepts_turboquant2` passed, 1 test.
- Focused rerun: `turboquant2_sidecar_uses_compact_qjl_payload` passed, 1 test.
- Static check: `cargo check -p ecaz --lib --no-default-features --features pg18` passed.
- Whitespace check: `git diff --check` passed.

## Cleanup Scope

- Removed the validation-only reduced-dimension `turboquant2_768` format from the production-facing IVF reloption surface by reverting `0b3fd57f7`.
- Preserved Task 124 packet 037 evidence in history and documentation.
- Added ignore rules for `reviews/**/truth-*.json` and `benchmarks/**/truth-*.json`.
- Did not delete local untracked truth caches. They are now ignored, and deletion can be handled separately if the operator wants local workspace pruning.
