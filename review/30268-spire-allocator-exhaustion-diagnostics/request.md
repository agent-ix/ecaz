# Review Request: SPIRE Allocator Exhaustion Diagnostics

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `362e6ad4 Add SPIRE allocator exhaustion diagnostics`

## Scope

This packet covers a small architecture follow-up cleanup: PID and local vec_id
allocators now expose non-mutating near-exhaustion diagnostics.

Changed files:

- `src/am/ec_spire/assign.rs`
- `src/am/ec_spire/diagnostics.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added `SpireAllocatorExhaustionDiagnostics` with:
  - next value
  - remaining successful allocations before exhaustion
  - threshold-based near-exhaustion flag
- Added `exhaustion_diagnostics(warn_within)` on PID and local vec_id
  allocators.
- Added `collect_allocator_diagnostics` to derive PID and local vec_id status
  from root/control cursors without advancing either sequence.
- Added tests for PID/local vec_id near-exhaustion status and root/control
  diagnostics.
- Updated the Task 30 notes and architecture feedback response.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `181 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

Allocator exhaustion still fails closed at allocation/observe time. This slice
adds an inspectable warning surface for diagnostics and future admin views; it
does not alter allocation semantics.
