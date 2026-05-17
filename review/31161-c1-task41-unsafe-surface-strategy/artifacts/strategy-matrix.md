# Task 41 Unsafe Surface Strategy Matrix

## Relation Resources

Current wrapper: `AccessShareIndexRelation`.

Primary target:

- `src/lib.rs`: remaining raw `open_valid_ec_*_index` callers.
- `src/tests/*`: raw helper callers only where they block deletion of the raw
  helpers.

Action:

- keep migrating production entrypoints to guard-owning code,
- add a validation-only helper for callers that only need AM/relkind checks,
- treat `AccessShareIndexRelation::as_ptr()` as a borrow only; the guard must
  remain live across every raw relation read, and graph-loader callers must keep
  the guard alive until after the final load completes,
- prefer re-opening short guards over one long guard when the long guard would
  span environment-variable lookups, FFI calls into non-PostgreSQL code, SPI
  work, or large control-flow regions,
- delete raw `open_valid_ec_hnsw_index`, `open_valid_ec_ivf_index`,
  `open_valid_ec_spire_index`, and `open_valid_ec_diskann_index` after the last
  caller migrates.

Status notes:

- `AccessShareIndexRelation::into_raw` is deleted. Guard callers can borrow the
  raw relation pointer, but cannot transfer relation-close ownership out of the
  guard API.
- Raw HNSW, IVF, SPIRE, and DiskANN `open_valid_ec_*_index` compatibility
  helpers are gone; remaining helper names with a `_guard` suffix return an
  owning guard.
- Remaining relation-resource work is direct PostgreSQL open/close use in SPIRE
  modules and other AM internals, not the old raw helper surface.

Projected baseline after relation-resource closeout: roughly 4,200-4,300 from
the current 4,321 head, depending on how many SPIRE direct opens collapse into
shared guards.

## Buffer, Page, and WAL Resources

Primary targets:

- `src/am/ec_ivf/page.rs`
- `src/am/ec_spire/page.rs`
- `src/storage/wal.rs`
- AM scan/build/insert/vacuum modules that pair buffer pins, locks, generic
  WAL state, and tuple/page pointers.

Action:

- create narrow RAII wrappers for pin/release, lock/unlock, and
  start/finish/abort pairs before adding comments,
- keep raw page pointer arithmetic behind typed page-reader/writer functions,
- add Task 43 coverage before declaring residual pointer contracts stable.

First slice: introduce the narrow buffer pin/lock guard in the lowest shared
page helper that covers `src/am/ec_ivf/page.rs`, then migrate a small contiguous
callsite group before expanding to SPIRE pages or WAL.

Projected baseline after the first buffer/page/WAL pass: roughly 3,700-3,900 if
the main paired resource sites are wrapped before residual pointer arithmetic is
left for Task 43.

## Scan Descriptor, Tuple Slot, and Detoast Lifetimes

Primary targets:

- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/scan_debug.rs`
- `src/am/ec_spire/scan/relation.rs`
- custom scan modules under `src/am/ec_spire/custom_scan/`.

Action:

- prefer owned scan-state containers over repeated raw descriptor access,
- isolate tuple slot allocation/drop and detoast ownership in wrappers,
- use Task 43 to exercise repeated rescan/end-scan and panic/error paths.

Projected baseline after scan/slot/detoast wrappers: roughly 3,300-3,500, with
the remaining scan unsafe concentrated in descriptor field access and Task
43-backed pointer lifetime contracts.

## Synchronization and Parallel State

Primary targets:

- `src/am/ec_hnsw/shared.rs`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/build_parallel.rs`

Action:

- defer broad annotation until Task 40 lifts the primitives,
- only patch immediate correctness bugs or resource leaks found during review.

Projected baseline after Task 40-owned sync lift: roughly 3,100-3,300.

## SIMD and Quantization

Primary targets:

- `src/quant/hadamard.rs`
- `src/quant/grouped_pq.rs`
- `src/quant/prod.rs`
- `src/quant/rabitq.rs`

Action:

- keep intrinsics in small target-feature modules,
- verify alignment, length, and target-feature contracts with Task 43 lanes
  before Task 35 residual comments.

Projected baseline after Task 43 proof lanes and SIMD cleanup: roughly
2,900-3,100. This track is not expected to delete all intrinsic unsafe; it
should leave only small, proved blocks with SAFETY comments.

## Test-Only Unsafe

Primary targets:

- `src/tests/ec_ivf.rs`
- `src/tests/ec_hnsw_scan_gettuple.rs`
- `src/tests/ec_hnsw_recall_debug_exports.rs`
- `src/tests/mod.rs`

Action:

- do not let test-only counts steer production sequencing,
- migrate tests when they block deleting production raw helpers,
- otherwise handle after production wrapper tracks shrink.

Projected baseline after test-only sweep: roughly 2,400-2,700, with remaining
entries expected to be production residuals requiring Task 35 comments.

## Task 41 Closeout

Task 41 closes when all paired PostgreSQL resource surfaces that can reasonably
be wrapped have RAII ownership, the raw compatibility helpers are deleted, and
the residual baseline is restricted to:

- Task 40 synchronization lifts,
- Task 43 proof-backed pointer, scan, page, and SIMD contracts,
- genuinely irreducible unsafe blocks ready for Task 35 SAFETY comments.
