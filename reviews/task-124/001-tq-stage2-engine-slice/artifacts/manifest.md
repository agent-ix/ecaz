# Artifact Manifest

Head SHA: `008309ae4` before local Task 124 edits.
Task bucket: `reviews/task-124/`
Packet path: `reviews/task-124/001-tq-stage2-engine-slice/`
Timestamp: `2026-06-27T22:22:07Z`
Lane / fixture / storage format / rerank mode: local dev, focused unit + PG18 tests, `ec_ivf` `coarse_rerank`, `rerank_format = 'turboquant'`, `rerank_placement = 'index'`, `stage2_final_rerank_width = 3` for runtime fixture.
Isolation: focused one-index-per-test-table surfaces; no shared benchmark tables.

## Artifacts

- `tq-score-surface-audit.md`
  - Command: static source audit using `rg` and local file reads.
  - Key lines cited: Task 124 TQ hot path uses `score_turboquant_batch_from_payload_refs` from the index-side sidecar, through the `candidate_batch` scoring surface; exact-dequant TQ remains scalar/off-path.

- `cargo-test-ec-ivf-options.log`
  - Command: `cargo test -p ecaz am::ec_ivf::options > reviews/task-124/001-tq-stage2-engine-slice/artifacts/cargo-test-ec-ivf-options.log 2>&1`
  - Key result: `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 2197 filtered out`.

- `cargo-test-ec-ivf-scan.log`
  - Command: `cargo test -p ecaz am::ec_ivf::scan > reviews/task-124/001-tq-stage2-engine-slice/artifacts/cargo-test-ec-ivf-scan.log 2>&1`
  - Key result: `test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 2194 filtered out`.

- `cargo-pgrx-test-pg18-tq-stage2-final-escalated.log`
  - Command: `cargo pgrx test pg18 test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads > reviews/task-124/001-tq-stage2-engine-slice/artifacts/cargo-pgrx-test-pg18-tq-stage2-final-escalated.log 2>&1`
  - Key result: `test tests::pg_test_ec_ivf_tq_stage2_final_exact_width_bounds_heap_reads ... ok`; `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2222 filtered out`.
  - Runtime fixture: one-list `coarse_rerank` index with `rerank_format='turboquant'`, `rerank_placement='index'`, `rerank_width=8`, `stage2_final_rerank_width=3`.
  - Assertion summary: TQ sidecar payload bytes are scored, TQ borrowed batch path avoids survivor slab copy, final exact heap/source bytes equal `3 * dims * 4`, and emitted outputs are truncated to final width 3.

- `cargo-pgrx-test-pg18-tq-stage2-final.log`
  - Command: same focused pgrx command before escalation.
  - Key result: sandbox failure only, `Operation not permitted` writing to `/opt/homebrew/share/postgresql@18/extension/ecaz.control`.
  - Status: superseded by `cargo-pgrx-test-pg18-tq-stage2-final-escalated.log`.

## Benchmark Status

No `ecaz bench suite` benchmark evidence is included in this packet. This is an engine/API checkpoint only, not a Task 124 closeout or promotion claim. The task remains open pending Phase 3 counters and the required 10k / 50k / 100k A/B benchmark matrix.
