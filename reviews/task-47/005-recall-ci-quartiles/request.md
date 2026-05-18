# Review Request: Recall Confidence And Query Percentiles

Task: `plan/tasks/47-recall-and-cost-model-gates.md`

Implementation commit: `b47398b4318bf70a0307d2fda649e7fa4fadb5a0`

## Scope

This slice adds the reporting side of Task 47 checklist item 8 and part of item 20:

- `ecaz bench recall` still emits the legacy `recall@k` field used by existing gate thresholds.
- Recall tables now also include `queries`, `recall_trials`, `recall_ci95_low`, `recall_ci95_high`, `recall_p10`, `recall_p50`, and `recall_p90`.
- The confidence interval is a Wilson 95% interval over `queries * k` hit/miss trials.
- Per-query recall percentiles count missing prediction rows as zero-recall queries, preserving worst-case visibility.
- Suite table parsing is covered for the expanded recall table shape.
- `docs/recall-floors.md` documents the new fields and states that hard-thresholding on `recall_ci95_low` is deferred until calibration, because current floors came from single observed runs.

## Validation

Packet-local evidence is under `artifacts/`; see `artifacts/manifest.md`.

- `cargo test -p ecaz-cli recall_summary -- --test-threads=1`: 2 passed.
- `cargo test -p ecaz-cli parses_recall_result_table -- --test-threads=1`: 1 passed.
- `cargo check -p ecaz-cli`: passed.
- `git diff --check` for touched files: clean.

## Remaining Task 47 Gaps

This packet does not close all of Task 47. Remaining gaps include committed exact-KNN fixture caches/regeneration, mixed and real gate corpora, calibrated CI-aware floors, expanded DiskANN/real/per-node cost rows, `--accept-drift` audit, deliberate negative fixtures, and cross-arch variance evidence.
