# Review Request: Duplicate Bind Overflow + Scan Expansion (Phase 7I)

Branch: `adr034-diskann-rebased`
Author: coder-2
Target: `src/am/ec_diskann/insert.rs`,
`src/am/ec_diskann/routine.rs`

## What this packet is

This is the next narrow insert slice after 11040's full-backlink
rewrite replan.

It lands the missing duplicate-vector path for `ec_diskann`:

1. bind a new heap row to an existing live node instead of erroring
2. persist duplicate heap tids through an overflow tuple chain
3. expand scan results back into heap-row multiplicity at `amrescan`

Before this slice, a true duplicate still hit the Phase 7 boundary
error. After this slice, duplicate inserts succeed and ordered scans
return all duplicate heap rows.

## Why this slice

Phase 7 had one major user-visible gap left after 11040:

- unique inserts could append nodes, install backlinks, and survive
  full-slice rewrite
- true duplicates still errored instead of binding to the existing node

ADR-046's frozen rule 2 also explicitly calls out overflow-heaptid
chain growth as part of the ordered insert protocol:

- append-like allocation first
- ordered rewrites second
- metadata last

That makes duplicate bind the next clean vertical slice.

## What changed

### `insert.rs`

Added a local overflow-tuple codec and duplicate-bind write path:

- **`TQ_VAMANA_OVERFLOW_TAG = 0x08`**
- **`VamanaOverflowTuple`**
- **`bound_heap_tids_for_owner(...)`**
- **`bind_duplicate_heap_tid(...)`**
- **`cmp_item_pointer_physical(...)`**

The new overflow tuple is deliberately local to `insert.rs`. I did not
edit `tuple.rs` because this slice only needs:

1. a private codec for duplicate-bind persistence
2. a private owner-scan helper for scan-time expansion

The slim Vamana node layout still stays unchanged:

- `primary_heaptid`
- `has_overflow_heaptids`
- `rerank_tid = INVALID`

#### Overflow tuple shape

The new tuple carries:

- `owner_tid`
- `nexttid`
- `heap_tid_count`
- fixed-capacity `heap_tids`

Capacity is `HEAPTID_INLINE_CAPACITY` (`10`) so the tuple has fixed
encoded length and is safe for in-place patching.

#### Duplicate bind protocol

`bind_duplicate_heap_tid(...)` now does:

1. materialize the current chain
2. read the duplicate target node
3. gather that node's overflow tuples in physical `(block, offset)`
   order
4. no-op if the new heap tid is already bound
5. if the tail overflow tuple still has room, patch it in place
6. otherwise append a fresh overflow tuple under the append-like path,
   then patch the predecessor `nexttid`
7. patch the node's `has_overflow_heaptids = true` when the duplicate
   chain becomes live
8. retry from fresh state up to `MAX_BACKLINK_REPLAN_PASSES`

This follows ADR-046's split:

- append-like allocation first
- ordered rewrites afterward

#### Important runtime note

Current runtime duplicate expansion does **not** walk from a head
pointer, because the slim node tuple only stores a boolean
`has_overflow_heaptids` flag and no overflow-head TID.

So the read path currently discovers duplicates by:

- scanning raw tuples for matching `owner_tid`
- keeping physical tuple order
- expanding the primary heap tid first, then overflow heap tids

`nexttid` is still written on overflow growth so a future head-based
consumer or repair path has consistent chain links, but the scan path
does not depend on that link today.

### `routine.rs`

`ec_diskann_aminsert` now calls the real duplicate bind helper instead
of erroring on the true-duplicate path.

`ec_diskann_amrescan` now expands each node-level `ScanResult` back into
heap-row multiplicity through:

- **`expand_scan_results_with_bound_heap_tids(...)`**

That helper:

1. takes the node-level results from `vamana_scan_with(...)`
2. expands each node into `primary_heaptid + overflow heap tids`
3. preserves the exact rerank distance for every duplicate row of that
   node
4. truncates the final buffer to `opaque.top_k`

`amgettuple` stays unchanged: it still drains `opaque.result_buf`, but
that buffer now contains one entry per heap row rather than one entry
per node when duplicates exist.

### Shared cleanup

I also took the reviewer suggestion from 11038 to collapse the repeated
physical item-pointer ordering into one shared helper:

- **`insert::cmp_item_pointer_physical(...)`**

`routine.rs` now reuses that comparator for sorted duplicate/backlink
target lists.

## Boundary after this packet

Non-empty insert now handles both major outcomes:

- true new-node append
- true duplicate bind

Insert now does:

- derive persisted payload
- detect true duplicates by exact heap-vector equality
- bind duplicate heap tids onto an existing live node
- grow overflow tuples when needed
- append unique nodes
- repair backlinks, including full-slice rewrite replan
- increment `inserted_since_rebuild` only for true new nodes

Insert still does **not**:

- maintain or repair the entry point
- let insert own `needs_medoid_refresh`
- add any cold rerank payload writes (`rerank_tid` stays `INVALID`)

## Tests

New pg coverage:

- **`test_ec_diskann_duplicate_insert_binds_first_overflow_tuple`**
  proves a second identical row binds to the seed node, sets the
  overflow flag, keeps `inserted_since_rebuild == 1`, and returns both
  ids through runtime ordered scan
- **`test_ec_diskann_duplicate_after_append_binds_existing_node`**
  proves duplicate bind works against an appended live node, not just
  the bootstrap node
- **`test_ec_diskann_duplicate_bind_grows_second_overflow_tuple`**
  forces `12` identical rows so the overflow path has to spill past one
  fixed-capacity tuple; runtime ordered scan returns all `12` ids

Retained coverage:

- full `ec_diskann` pure-Rust + pg test suite
- full repo `cargo test`
- full pg17 script

## Verification

```text
cargo fmt -- src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs
cargo build --lib
cargo clippy --lib --no-deps
cargo test --lib ec_diskann
cargo test
bash scripts/run_pgrx_pg17_test.sh
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

Observed:

- `cargo fmt -- src/am/ec_diskann/insert.rs src/am/ec_diskann/routine.rs`
  — passed
- `cargo build --lib` — passed
- `cargo clippy --lib --no-deps` — passed with only the known baseline
  `unnecessary_sort_by` warnings in untouched `reader.rs`, `scan.rs`,
  and `vamana.rs`
- `cargo test --lib ec_diskann` — passed with `134 passed`, `0 failed`
- `cargo test` — passed
- `bash scripts/run_pgrx_pg17_test.sh` — passed
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  — still fails only in untouched baseline code/tests:
  - existing `reader.rs`, `scan.rs`, and `vamana.rs` sort warnings
  - existing `scan.rs` test-only warnings
  - existing `vacuum.rs` test-only warning

## Reviewer notes

- **Duplicate identity is still exact heap-vector equality.** Payload
  equality only narrows the candidate set; the final duplicate verdict
  remains exact `ecvector` equality from the heap.
- **This slice intentionally keeps overflow discovery owner-based.**
  The slim node layout exposes only `has_overflow_heaptids`, not a head
  pointer. Physical owner scan is the narrowest way to ship duplicate
  bind without reopening ADR-045's node layout.
- **`nexttid` is maintained, but not yet load-bearing at runtime.**
  Growth patches the predecessor link on the happy path, but scan-time
  duplicate expansion does not rely on that link today.
- **`inserted_since_rebuild` remains true-new-node only.** Duplicate
  binds do not advance it and still leave `needs_medoid_refresh`
  untouched.

## Not doing in this packet

- **Entry-point maintenance**
- **Insert ownership of `needs_medoid_refresh`**
- **Cold rerank payload writes**
- **Any change outside `src/am/ec_diskann/`, `review/`, or packet docs**
