# Review Request: SPIRE Explicit Dedupe Mode

Status: open
Branch: `task30-spire-partition-object-spec`
Checkpoint commit: `53d2267c Add SPIRE explicit dedupe mode`

## Scope

This packet covers the A7 pre-persistence architecture feedback slice: make
candidate dedupe explicit so the Phase 1 primary-only local path does not pay
for `vec_id` dedupe until boundary replicas or remote merge require it.

Changed files:

- `src/am/ec_spire/options.rs`
- `src/am/ec_spire/scan.rs`
- `plan/tasks/30-spire-ivf-foundation.md`
- `plan/design/spire-foundation-architecture-feedback-response.md`

## What Changed

- Added `SpireCandidateDedupeMode`:
  - `NoReplicaDedupeDisabled`
  - `VecIdDedupeEnabled`
- Added `dedupe_mode` to `SpireSingleLevelScanPlan`.
- Defaulted resolved Phase 1 scan plans to `NoReplicaDedupeDisabled`.
- Changed routed scan candidate ranking to allocate and use the
  `HashMap<SpireVecId, ...>` only when `VecIdDedupeEnabled` is selected.
- Kept the explicit dedupe path available for boundary replicas, retained
  mixed-ID epochs, and future remote candidate merge.
- Added regression coverage proving duplicate `vec_id` rows are retained when
  dedupe is disabled, while the existing dedupe-enabled helper path still keeps
  the best visible candidate.
- Updated the Task 30 plan and architecture feedback response note to mark the
  explicit dedupe-mode gate item complete.

## Validation

- `cargo fmt`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - Result: `173 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
- `git diff --cached --check`

Known formatting warning remains unchanged from prior checkpoints: stable
rustfmt reports that `imports_granularity` and `group_imports` require nightly.

## Review Notes

This checkpoint intentionally does not introduce boundary replicas. It only
makes the scan-time behavior explicit and keeps the opt-in dedupe path available
for the future replica and remote-merge work described by ADR-049.
