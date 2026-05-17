# Review Request: Task 28 IVF Empty Index

Scope: Phase 1 empty-index behavior. Makes `CREATE INDEX ... USING ec_ivf`
on empty relations produce a valid index metadata page and makes the scan
callbacks behave as a no-row AM path after rescan.

Task: `plan/tasks/28-ivf-access-method.md` Phase 1

Branch: `task28-ivf`

Head SHA: `7a776b7fb79313f034ea7565ff2dd3bc26259e19`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `git diff --cached --check`
- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Summary

This slice turns the IVF scaffold into a valid empty-index access method path:

- Empty `ec_ivf` builds initialize block 0 as a versioned metadata page.
- Metadata preserves validated IVF reloptions: `nlists`, `nprobe`,
  `training_sample_rows`, `seed`, `storage_format`, and `rerank`.
- Persisted storage/rerank enum codes now use explicit `u8` discriminants.
- Populated builds still fail loudly on the first heap tuple until centroid
  training and posting-list storage land.
- `ambeginscan`, `amrescan`, `amgettuple`, and `amendscan` now allocate,
  validate, return no rows for empty indexes, and free scan-local opaque state.
- The no-row path intentionally does not mutate order-by output slots.
- PG regression coverage verifies `ec_ivf` AM/opclass registration, metadata
  initialization, and heap-backed empty scan behavior.

## Review Focus

Please review for:

- Whether the metadata special-area layout is a sound first disk contract.
- Whether the metadata magic/version and explicit enum code assignments are
  adequate before adding posting-list pages.
- Whether `table_index_build_scan` plus a first-row error is the right
  temporary populated-build boundary.
- Whether scan validation should keep the current strict `nkeys = 0` and
  `norderbys = 1` contract until real candidate scoring lands.
- Whether the empty false-return path is correct to leave order-by output
  storage untouched.
- Whether the heap-backed debug helper uses snapshots and relation locks
  correctly for regression coverage.
- Whether `ec_ivf.nprobe` should stay exposed now even though the runtime
  override is not consumed until real scans exist.

## Non-Goals

This packet does not implement centroid training, posting-list tuple layout,
quantized candidate scoring, live insert, vacuum repair, recall validation, or
planner costing for `ec_ivf`.
