# Review Request: AM Routine Unsafe Boundary Reduction

Head: `57e9f44929d758a9f8d308c8b62c366e7f30f67a`

Scope:
- `scripts/unsafe_comment_baseline.txt`
- `src/am/ec_diskann/diagnostics.rs`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_hnsw/routine.rs`
- `src/am/ec_ivf/routine.rs`
- `src/am/ec_spire/routine.rs`

What changed:
- Removed unnecessary unsafe blocks from all four `amvalidate` callbacks.
  These callbacks now return `true` directly instead of entering
  `pgrx_extern_c_guard` for a constant expression.
- Removed duplicated explicit `pgrx_extern_c_guard` calls from the HNSW, IVF,
  SPIRE, and DiskANN handler bodies. The handlers already use `#[pg_guard]`,
  which pgrx documents as the intended guard for PostgreSQL `extern
  "C-unwind"` callback boundaries.
- Left comments on the handler bodies to make the boundary handling explicit.
- Documented the two remaining DiskANN graph diagnostic unsafe reads, which
  still depend on a live PostgreSQL relation pointer.

Baseline result:
- Start: 4,809 entries across 117 files.
- End: 4,799 entries across 113 files.
- Net reduction: 10 entries and 4 files.
- Handling split: 8 unsafe blocks removed, 2 remaining unsafe blocks documented.

Review focus:
- Confirm relying on `#[pg_guard]` is the correct replacement for the removed
  inner `pgrx_extern_c_guard` calls.
- Confirm comment-only handling is justified for the two DiskANN diagnostic
  reads because they cross the PostgreSQL relation/page reader boundary.
- Confirm the baseline changed in the intended direction despite line-number
  churn in `src/am/ec_diskann/routine.rs` after removing handler lines.

Validation:
- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before-902.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`

Notes:
- pgrx 0.17 documents `pgrx_extern_c_guard` as normally unnecessary when
  `#[pg_guard]` is present, and says direct use should be limited to top-level
  `extern "C-unwind"` functions. That matches the handler change here.
- `cargo check` passed with existing warnings from PostgreSQL headers and
  currently unused SPIRE re-exports in `src/am/mod.rs`.
