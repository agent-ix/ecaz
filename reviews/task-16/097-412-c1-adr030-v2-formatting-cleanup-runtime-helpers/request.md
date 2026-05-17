# Review Request: C1 ADR-030 V2 Formatting Cleanup Runtime Helpers

Current head: `b307b8a`

## Context

Three tracked files were left locally modified after the recent `pq_fastscan`
 runtime-helper and vacuum slices:

- `src/am/scan_debug.rs`
- `src/am/shared.rs`
- `src/am/vacuum.rs`

Those diffs were pure formatting spillover from `rustfmt`, not intentional
behavior changes. Leaving them uncommitted made the branch look dirtier than it
really was.

## Problem

Before this slice:

1. the branch head was pushed cleanly
2. but these three files still had local formatting-only diffs
3. that made it ambiguous whether there was uncommitted runtime logic

The right fix was to package the formatting cleanup explicitly rather than leave
it as local residue.

## Implementation

Updated:

- `src/am/scan_debug.rs`
- `src/am/shared.rs`
- `src/am/vacuum.rs`

All changes are formatting-only:

1. line wrapping / indentation around `unwrap_or_else(...)`
2. line wrapping in small `match` expressions inside vacuum tests
3. no semantic or API changes

## Validation

Passed:

- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Required full-test commands were run and hit the same known workstation linker
boundary as the rest of this branch:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Observed unresolved PostgreSQL symbols remained in the same family:

- `CurrentMemoryContext`
- `PG_exception_stack`
- `error_context_stack`
- `CopyErrorData`
- `errstart`

## Outcome

This is strictly a hygiene checkpoint:

1. the formatting spillover is now committed instead of lingering locally
2. the branch diff once again reflects only intentional uncommitted work
3. runtime behavior is unchanged
