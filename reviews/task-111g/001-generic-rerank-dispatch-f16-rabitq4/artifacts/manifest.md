# Packet 001 — artifact manifest

- Task bucket: `reviews/task-111g/`
- Packet: `001-generic-rerank-dispatch-f16-rabitq4`
- Branch: `task-111g-coarse-rerank-representations`
- Head SHA: `aad8c3ccff7e5d2ae90adc081b4c3a3975d0db30`
- Lane: local Intel desktop (PG18, pgrx managed instance)
- Storage format under test: `coarse_rerank` (RaBitQ-1 coarse stage, table-side rerank)
- Rerank formats under test: `f32`, `f16`, `rabitq4`
- Surfaces: isolated one-index-per-table fixtures (each fixture builds its own table + index)
- Code change: yes (under review in this packet) — no benchmark numbers here

## Commands

- Static:
  - `cargo check --no-default-features --features pg18`
  - `cargo clippy --no-default-features --features pg18 -- -D warnings`
- Unit:
  - `cargo test --no-default-features --features pg18 --lib am::ec_ivf::options`
  - `cargo test --no-default-features --features pg18 --lib am::ec_ivf::rerank`
- pg_test (PG18):
  - `cargo pgrx test pg18 --no-default-features --features "pg18 pg_test" coarse_rerank`
  - `cargo pgrx test pg18 --no-default-features --features "pg18 pg_test" heap_f32_rerank`

## Artifacts and the key result lines `request.md` cites

| File | What it shows | Key lines |
| --- | --- | --- |
| `unit-rerank.log` | rerank.rs unit tests (f16 round-trip, f32 equivalence) | `test result: ok. 7 passed; 0 failed` |
| `unit-options.log` | options.rs unit tests (accept f16/rabitq4, reject rabitq2/8/tq + index placement) | `test result: ok. 11 passed; 0 failed` |
| `pgtest-coarse-rerank.log` | PG18 coarse_rerank fixtures | `pg_test_ec_ivf_coarse_rerank_f16_matches_f32_ranking ... ok`; `..._rabitq4_returns_correct_top_neighbor ... ok`; `..._f16_rabitq4_admin_snapshot ... ok`; `test result: ok. 15 passed; 0 failed` |
| `pgtest-heap-f32-regression.log` | PG18 heap_f32 rerank regression (AC1 bit-identical proof) | `pg_test_ec_ivf_heap_f32_rerank_full_probe_matches_exact_scores ... ok`; `test result: ok. 3 passed; 0 failed` |

Timestamp: 2026-06-18 (local run on this branch head).

## Notes

- The pgrx test runner prints the lib `pg_test` results first; the cited logs
  were grep-filtered to the relevant `Running unittests src/lib.rs` section plus
  the `test result` summary lines.
- No corpus TSVs, SSM exhaust, or poll snapshots are committed (per repo policy);
  fixtures use tiny inline deterministic corpora generated in-test.
