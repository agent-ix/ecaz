# Review Request: Vacuum Stale-Rewrite Replan (Phase 8B-3)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/routine.rs`

## What this packet is

This is the next `ec_diskann` vacuum slice after packet `11043`'s
fill-only repair boundary.

It closes the remaining ADR-047 retry gap for V0:

1. stale tuple bytes during vacuum writeback are no longer a hard error
2. the callback now replans from fresh persisted state up to a bounded
   cap
3. the new pg regression proves the retry path actually runs

Before this slice, any tuple drift between the read-only repair plan and
the page-local rewrite would abort the vacuum call. After this slice,
vacuum retries the whole ordered repair pass instead.

## Why this slice

Packet `11043` made pass 2 structurally stronger, but it still had one
load-bearing correctness gap:

- vacuum planned against a materialized chain snapshot
- insert could still mutate one of those target tuples before writeback
- `apply_tuple_rewrites(...)` treated that drift as an immediate error

ADR-047 review answer `G5` called out the intended closeout explicitly:

- named retry cap: `MAX_REPAIR_REPLAN_PASSES = 3`
- loud failure only after the cap is exhausted

That is the narrowest safe next slice because it fixes the mutation race
without changing the existing fill-only write contract.

## What changed

### `routine.rs`

Added:

- **`MAX_REPAIR_REPLAN_PASSES: usize = 3`**
- **`VacuumRewriteApplyOutcome::{Applied, RetryReplan}`**
- **`VacuumBulkDeletePassResult`**
- **`run_diskann_bulkdelete_pass(...)`**
- **`chain_entry_point_needs_medoid_refresh(...)`**

`run_diskann_bulkdelete(...)` is now an outer retry loop:

1. run one full ordered pass against a fresh materialized chain
2. if writeback finishes cleanly, publish stats and return
3. if any tuple requested replanning, rematerialize and retry
4. error only after `MAX_REPAIR_REPLAN_PASSES`

`apply_tuple_rewrites(...)` now distinguishes:

- **hard errors**: corrupt bounds, invalid tuple lengths, buffer/open
  failures
- **retryable drift**: reopened tuple bytes no longer match the planned
  `expected_raw`

That means stale rewrite state is now a control-flow signal, not a fatal
vacuum error.

#### Metadata ownership fix

`needs_medoid_refresh` is now derived from the final materialized chain
state through `chain_entry_point_needs_medoid_refresh(...)`, not from a
"did this pass just finalize the entry point" boolean.

That matters under retry: an earlier partial pass may already have
tombstoned the entry point before a later block requests replanning.
This slice keeps the metadata flag monotonic across that case.

#### Stats under retry

`tuples_removed` is tracked as the max removed-heap-tid count seen across
retry passes.

That keeps the reported removal count stable even when an earlier pass
already committed some pass-1 owner cleanup before a later block
requested replanning.

### Test-only support

Under `#[cfg(any(test, feature = "pg_test"))]`, `routine.rs` now has a
small rewrite-injection seam plus a retry counter so pg tests can force
exactly one stale-write race without depending on timing.

This is test-only and does not change production control flow.

### pg tests

Added:

- **`find_vacuum_refill_fixture(...)`**
  factors the existing refill-topology search into a reusable helper
- **`test_ec_diskann_vacuum_replans_on_stale_repair_tuple`**
  installs a one-shot on-disk drift injection during vacuum writeback
  and proves:
  - vacuum succeeds instead of erroring
  - exactly one retry pass was taken
  - the deleted neighbor stays gone
  - the replacement neighbor is preserved after replanning

The existing refill test now reuses the same fixture helper.

## Boundary after this packet

`ec_diskann` vacuum now supports:

- duplicate-safe pass 1 heap-tid stripping
- pass 2 unlink + fill-only repair
- bounded stale-write replanning with cap `3`
- pass 3 tombstoning of fully-dead nodes
- monotonic `needs_medoid_refresh` ownership under retry

`ec_diskann` vacuum still does **not** support:

- live-neighbor eviction under the write lock
- any cold rerank chain cleanup (correctly absent in V0)
- planner/cost activation work (still Phase 9)

So this packet closes the V0 ADR-047 retry boundary without widening the
write contract beyond fill-only repair.

## Tests

New coverage:

- **`test_ec_diskann_vacuum_replans_on_stale_repair_tuple`**

Retained relevant coverage:

- **`test_ec_diskann_vacuum_refills_broken_neighbor_slot`**
- full `ec_diskann` unit + pg-test surface
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo fmt -- src/am/ec_diskann/routine.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/routine.rs` — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `140 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only on the untouched baseline:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only `unnecessary_cast` /
    `needless_borrows_for_generic_args`
  - existing `vacuum.rs` test-only `needless_range_loop`

## Reviewer notes

- **Retry is whole-pass, not tuple-local.** The callback rematerializes
  the persisted chain and reruns ordered repair from current on-disk
  state.
- **Partial earlier-page writes are allowed.** If a later block asks for
  replanning, the next pass starts from the already-committed earlier
  changes.
- **Write behavior is still fill-only.** The retry path does not grant
  vacuum any new live-neighbor eviction power.
- **The new pg regression is deterministic.** It uses a one-shot
  rewrite injection and asserts the retry counter, so it does not depend
  on thread timing.

## Not doing in this packet

- **Any Phase 9 planner/cost work**
- **Any `ec_diskann` strict-clippy baseline cleanup outside touched code**
- **Any change outside `src/am/ec_diskann/`, `review/`, or packet docs**
