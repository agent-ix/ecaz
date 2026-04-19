# Review Request: C1 Native Build Stability Tightening

Current head at execution: `2301fca`

## Context

This checkpoint follows the native-build landing and the cleanup packet `449`.
It addresses the reviewer concerns on the native serial builder that were cheap
to fix now and directly reduce risk before merge:

- remove redundant upper-layer descent work
- make build-time level selection respect the build page size explicitly
- turn silent neighbor-slot clamping into an invariant check
- document the current `ef_construction` choice on upper layers

This is still not a parallel-build slice and does not change persisted page or
tuple layout.

## What changed

### 1. Layer-0 search now reuses the upper-layer walk result

Previously native build:

- ran `populate_native_upper_layer_forward_slots(...)`, which already walks down
  from the entry level with successor search, then
- immediately ran a second `greedy_descend_with_successors(...)` from the
  original entry candidate before layer-0 search

That duplicated upper-layer descent work.

Now `populate_native_upper_layer_forward_slots(...)` returns both:

- the recorded forward-link selections
- the final seed frontier from the upper-layer walk

Layer-0 search uses those final seeds directly.

### 2. Build-time insert-level sampling now respects `state.page_size`

`insert.rs` now exposes:

- `choose_insert_level_for_page_size(...)`

The BUILD path uses that helper instead of always sampling against `BLCKSZ`.
This aligns native build with the actual page size carried by `BuildState`.

I also left a `debug_assert_eq!(state.page_size, BLCKSZ)` in the serial builder
so the current assumption stays visible while all in-repo build callers still
use normal PostgreSQL page sizing.

### 3. Slot-bound slicing no longer silently clamps

`load_native_successor_candidates(...)` used:

- `start.min(len)..end.min(len)`

That hid slot-layout drift by quietly truncating the neighbor slice.

It now:

- `debug_assert!`s that the computed layer end stays within the slot array
- slices with `start..end`

So the one real invariant is explicit instead of masked.

### 4. The upper-layer `ef_construction` choice is documented

I added a code comment at the upper-layer walk explaining that native build
intentionally uses the same `ef_construction`-width successor search on upper
layers as it does at layer 0. That makes the current recall/cost tradeoff
explicit until a later optimization slice changes it deliberately.

## Validation

Green checkpoint validation:

```bash
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Note: I had to rerun `cargo test` once in isolation after an initial parallel
run because the pgrx wrapper’s rebuild/install step invalidated the concurrently
running plain test lane. The final reported `cargo test` result here is from the
clean standalone rerun.

## Review focus

1. Is reusing the upper-layer terminal seeds for layer-0 search the right serial
   shape, or do you still want an explicit second descent for any reason?
2. Is the current `page_size` treatment good enough for this branch, or do you
   want the `debug_assert_eq!(..., BLCKSZ)` removed now that BUILD uses the
   page-size-aware helper?
3. Are the remaining native-build concerns now mostly in the “future tuning /
   measurement” bucket rather than merge blockers?
