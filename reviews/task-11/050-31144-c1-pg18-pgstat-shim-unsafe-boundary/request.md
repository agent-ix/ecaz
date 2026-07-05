# Review Request: PG18 pgstat Unsafe Boundary

## Summary

This slice centralizes the PG18 pgstat C ABI unsafe surface inside
`src/pg18_pgstat_shim.rs` and removes the corresponding unsafe requirements
from normal stats callers.

Code checkpoint: `1931ba8af2c849a7c472a1830d242d650186175b`

## Safety Handling

- Made the public Rust shim functions safe:
  - `register_kind()`
  - `is_registered()`
  - `record(&TqStatsCounters)`
  - `snapshot()`
- Kept the actual C FFI calls in short local unsafe blocks with SAFETY
  comments describing the C-side contract.
- Removed unsafe blocks from `src/am/common/stats.rs` for recording,
  registration, readiness checks, and snapshots.
- Made `register_pg18_stats()` safe because callers no longer need to uphold
  raw pointer or C ABI invariants.

The remaining unsafe is not deleted because crossing the C ABI still requires
unsafe Rust. The improvement is that the unsafe obligation is now handled in
one module instead of leaking into ordinary stats code.

## Baseline Delta

- Before: 4,795 unsafe baseline entries across 112 files.
- After: 4,787 unsafe baseline entries across 110 files.
- Net: 8 entries removed, 2 files removed from the unsafe baseline.

Removed baseline entries:

- `src/am/common/stats.rs:87`
- `src/am/common/stats.rs:97`
- `src/am/common/stats.rs:125`
- `src/am/common/stats.rs:191`
- `src/pg18_pgstat_shim.rs:19`
- `src/pg18_pgstat_shim.rs:29`
- `src/pg18_pgstat_shim.rs:39`
- `src/pg18_pgstat_shim.rs:50`

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `bash scripts/unsafe_baseline_report.sh`
- `make fmt-check`
- `git diff --check HEAD^ HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`

`cargo check` passes with the existing PostgreSQL header warnings and existing
unused SPIRE re-export warning.

## Artifacts

- `artifacts/unsafe-baseline-before.log`
- `artifacts/unsafe-baseline-after.log`
- `artifacts/audit-unsafe.log`
- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18.log`

## Review Focus

- Is the safe wrapper boundary in `src/pg18_pgstat_shim.rs` acceptable, or
  should any of the functions remain unsafe for callers?
- Are the C shim invariants in the SAFETY comments complete enough, especially
  for `record()` copying from `delta` and `snapshot()` writing to initialized
  Rust storage?
- Does keeping `_PG_init` unchanged avoid unnecessary churn without hiding any
  remaining pgstat unsafe obligation?
