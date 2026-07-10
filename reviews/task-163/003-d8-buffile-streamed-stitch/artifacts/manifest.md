# Task 163 packet 003 artifacts manifest

- **Head SHA:** `079a235f9433d52796107fdbc926c0ee5274940f`
- **Task bucket / packet:**
  `reviews/task-163/003-d8-buffile-streamed-stitch`
- **Branch:** `task-179-ec-distann-physical-shards`
- **Timestamp:** `2026-07-10T10:41:07-07:00`
- **Lane:** Task 163 D8 / FR-077-CON-4 implementation prerequisite
- **Fixture:** pure randomized shard suite plus focused live local PG18 pgrx
- **Storage format:** temporary per-shard PostgreSQL `BufFile`; no persisted
  index-format change
- **Rerank mode / corpus:** not applicable
- **Isolated one-index-per-table or shared-table surface:** focused tests create
  isolated ec_distann tables/indexes; no benchmark measurement is claimed

## Commands

```text
cargo test --lib am::ec_distann::shard_build::tests --no-default-features --features pg18
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
cargo pgrx test pg18 --no-default-features --features pg18 ec_distann_sharded_build
```

Each command was captured with `script -q -e` and `CARGO_TERM_COLOR=never` so
the packet preserves its command exit status and complete output.

## Artifacts

- `cargo-test-shard-build.log` — focused pure encoder/cursor/property suite.
- `cargo-clippy-pg18.log` — strict PG18 library clippy.
- `cargo-pgrx-test-sharded-build.log` — live PG18 self-recall and deterministic
  reindex tests through the installed extension and runtime `BufFile` path.

## Key result lines cited by `request.md`

- `test result: ok. 10 passed; 0 failed`
- `test ... tc038_d8_spill_and_cursor_bound ... ok`
- `Finished dev profile` from clippy with command exit code 0.
- `test ... pg_test_ec_distann_sharded_build_self_recall ... ok`
- `test ... pg_test_ec_distann_sharded_build_is_deterministic_across_reindex ... ok`
- Focused PG result: `2 passed; 0 failed`.

## Provenance notes

- The code checkpoint was committed before these artifacts were captured.
- This is correctness/memory-contract evidence, not a benchmark packet; there
  is no `results.jsonl` and no latency/recall/storage result is newly claimed.
- The extension PG tests compile `pg18 pg_test` and install the non-`cfg(test)`
  library, which selects the PostgreSQL `BufFile` implementation. The pure Rust
  test binary selects the in-memory transport only for backend-independent unit
  testing.
