# Task 111h / 001 — Placement Semantics Checkpoint

Code commit: `a3b6eda24e897484b6ef26bc4f4b463f6bc42d46`

## Summary

This checkpoint removes the misleading product-facing interpretation where
`rerank_placement = 'table'` meant "read the f32 heap source and convert/score it
as f16/rabitq4 at query time."

New semantics:

- `rerank_placement = 'source'`: uses the existing f32 heap/source vector. This
  is the exact f32 baseline and writes no rerank sidecar.
- `rerank_placement = 'index'`: uses the existing persisted `0x2A` compact sidecar
  for compact formats. This remains the legacy index-side baseline for 111h, not
  the final packed group/segment layout.
- `rerank_placement = 'source_diagnostic'`: explicit benchmark-only surface for
  the legacy query-time compact conversion path.
- `rerank_placement = 'table'`: now reserved for real table-owned persisted
  rerank payloads and errors until that design exists.

Auto placement now resolves to `source` for f32 and to `index` for currently
implemented compact formats (`f16`, `rabitq4`) so compact-format requests do not
silently become query-time source conversion.

## Code Changes

- `src/am/ec_ivf/options.rs`: adds `Source` and `SourceDiagnostic` placement
  variants, reserves `Table`, and updates option resolution/tests.
- `src/am/ec_ivf/scan.rs`: renames the heap-vector rerank helper/comments from
  table-side to source-side.
- `src/am/ec_ivf/rerank.rs`, `src/am/ec_ivf/page.rs`, `src/am/ec_ivf/build.rs`,
  `src/am/ec_ivf/insert.rs`, `src/am/ec_ivf/vacuum.rs`, and
  `docs/on-disk-format.md`: update durable comments/docs to the new placement
  vocabulary.
- `src/tests/ec_ivf.rs`: updates pg_test fixtures so f32 uses `source`, compact
  auto-placement reports `index`, and the legacy f16 comparison uses
  `source_diagnostic`.
- `plan/tasks/111h-ivf-persisted-rerank-format-sweep.md`: marks the completed
  placement/diagnostic-gating checklist items.

## Validation

Artifacts are under `reviews/task-111h/001-placement-semantics/artifacts/`.

- `cargo test --no-default-features --features pg18 coarse_rerank --lib`
  passed under `script` capture:
  `21 passed; 0 failed; 0 ignored; 0 measured; 2171 filtered out`.

## Review Focus

- Confirm the new placement names and error behavior match Task 111h intent.
- Confirm `source_diagnostic` is acceptable as the explicit legacy benchmark
  surface for query-time compact conversion.
- Confirm auto placement resolving compact formats to the existing index sidecar
  is the right interim behavior until the packed rerank layout lands.
