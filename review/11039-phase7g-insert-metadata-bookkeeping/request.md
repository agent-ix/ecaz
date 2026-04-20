# Review Request: Insert Metadata Bookkeeping (Phase 7G)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow non-empty insert slice after 11038's
free-capacity backlinks.

It lands exactly one metadata rule from ADR-046 reviewer feedback:

1. true new-node inserts increment `inserted_since_rebuild`
2. duplicate insert attempts do not increment it
3. insert still does **not** set `needs_medoid_refresh`

Nothing else about neighbor repair, entry-point maintenance, or
full-target rewrite changes in this packet.

## Why this slice

The reviewer guidance on 11002 is explicit:

- live insert owns `inserted_since_rebuild += 1` for true new-node
  inserts
- live insert does **not** own `needs_medoid_refresh`

That is a clean bookkeeping seam that can land independently of the
remaining mutation complexity. It also gives later maintenance work a
monotonic insert-drift counter without introducing a second writer for
the medoid-refresh flag.

## What changed

### `insert.rs`

Added:

- **`increment_inserted_since_rebuild(index_relation) -> Result<u64,
  String>`**

The helper reuses `with_locked_metadata_page(...)` to:

1. lock and WAL-register the metadata page
2. `checked_add(1)` the existing `inserted_since_rebuild` counter
3. error on `u64` overflow rather than silently wrapping

It does **not** touch `needs_medoid_refresh`.

### `routine.rs`

`ec_diskann_aminsert` now calls
`insert::increment_inserted_since_rebuild(...)` only after the unique
insert path has:

1. appended the new node
2. applied the free-capacity backlinks

The duplicate path remains unchanged, so duplicate insert attempts still
fail at the duplicate-bind boundary without mutating the metadata
counter.

### pg tests

The packet extends the existing `routine.rs` pg tests with a small
metadata-reader helper that reopens the index relation and decodes the
metadata page directly through
`scan_state::materialize_chain_from_index(...)`.

New assertions:

- **bootstrap insert**: `inserted_since_rebuild == 1`,
  `needs_medoid_refresh == false`
- **second distinct insert**: `inserted_since_rebuild == 2`,
  `needs_medoid_refresh == false`
- **duplicate-after-append failure**: counter stays at `2`,
  `needs_medoid_refresh == false`

That proves both sides of the contract:

- true new-node inserts advance drift
- duplicate attempts do not

## Boundary after this packet

Non-empty unique insert now:

- derives persisted payload
- rejects true duplicates
- plans exact forward neighbors
- appends the new node
- writes free-capacity backlinks
- increments `inserted_since_rebuild`

Insert still does **not**:

- rewrite full target slices
- retry/replan after target drift
- update `needs_medoid_refresh`
- repair/promote the entry point
- grow overflow-heaptid chains

## Tests

Adjusted coverage:

- **`test_ec_diskann_empty_index_bootstrap_insert_executes`** now
  asserts bootstrap metadata bookkeeping
- **`test_ec_diskann_unique_insert_is_scan_reachable`** now also
  asserts the new-node metadata increment
- **`test_ec_diskann_duplicate_after_append_hits_boundary`** now proves
  duplicate attempts leave the metadata unchanged

Retained coverage:

- full `ec_diskann` pure-Rust and pg test suite
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `129 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only warnings
  - existing `vacuum.rs` test-only warning

The touched Rust files were also run through file-scoped `cargo fmt`
before verification.

## Reviewer notes

- **This is bookkeeping only.** Reviewers should not expect any new
  graph-mutation semantics beyond the already-landed Phase 7F behavior.
- **The medoid-refresh flag remains maintenance-owned.** That is
  deliberate and follows the 11002 feedback directly.
- **Tests read the metadata page directly.** I stayed inside the
  allowed `ec_diskann` files instead of extending the off-limits
  admin-snapshot SQL surface in `src/lib.rs`.

## Not doing in this packet

- **Full-target backlink rewrite**
- **Retry/replan loop**
- **Entry-point maintenance**
- **Overflow-heaptid growth**
- **Any insert-side write to `needs_medoid_refresh`**
