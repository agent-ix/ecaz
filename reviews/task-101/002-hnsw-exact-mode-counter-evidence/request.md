# Task 101 Packet 002: HNSW Exact-Mode Counter Evidence

## Summary

This checkpoint closes the interrupted Task 101 exact-mode counter slice.

Code change under review:
- `src/am/common/candidate_batch/mod.rs` now prevalidates TurboQuant no-QJL 4-bit code length before stripping MSE bytes for all three exact-mode batch helpers: `lut32`, `tiled_lut32`, and `int8_approx32`.
- A malformed mid-batch payload now rejects before score writes and before candidate/block counter records for all three modes.

Why this matters for Task 101:
- Task 101 acceptance criterion 3 requires the no-partial-write prevalidation contract to hold by construction across families.
- Packet 001 already added distinct counter kinds for `turboquant_tiled_lut` and `turboquant_int8`. This packet adds direct HNSW exact-mode evidence that those rows are visible in suite output and that recall remains stable.

## Validation

Manifest:
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/manifest.md`

Focused source check:
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/cargo-test-candidate-batch.log`
- Result: `19 passed; 0 failed; 2067 filtered out`
- New test covered: `turboquant_no_qjl_exact_modes_shape_error_scores_nothing_and_record_no_counters`

Suite evidence:
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/task101-hnsw-exact-mode-counter-suite.json`
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/suite-report.log`
- `reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/results-report.jsonl`
- Result: 15 suite steps completed, 0 failed, 0 skipped, 0 stale.

Key suite observations:
- full_lut recall is byte-equal across candidate-batch on/off: `recall@k=0.8375`, `ndcg@k=0.9872`
- tiled_lut recall is byte-equal across candidate-batch on/off: `recall@k=0.8375`, `ndcg@k=0.9872`
- int8_approx recall is byte-equal across candidate-batch on/off: `recall@k=0.8344`, `ndcg@k=0.9869`
- Direct exact-mode counter rows appear for:
  - `quant=turboquant`
  - `quant=turboquant_tiled_lut`
  - `quant=turboquant_int8`

## Review Focus

Please review:
- whether the shared `checked_mse_code_bytes_no_qjl_4bit` helper is the right narrow fix for the missed exact-mode prevalidation case;
- whether the new regression test proves the no-partial-score-write/no-counter contract for the three TurboQuant exact-mode helpers without overfitting to one driver shape;
- whether the packet evidence is sufficient for Task 101 AC3 and AC4 local HNSW exact-mode coverage.
