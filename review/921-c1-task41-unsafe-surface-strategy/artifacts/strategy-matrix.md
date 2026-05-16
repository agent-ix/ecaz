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
- delete raw `open_valid_ec_hnsw_index`, `open_valid_ec_ivf_index`,
  `open_valid_ec_spire_index`, and `open_valid_ec_diskann_index` after the last
  caller migrates.

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

## Synchronization and Parallel State

Primary targets:

- `src/am/ec_hnsw/shared.rs`
- `src/am/common/parallel.rs`
- `src/am/ec_hnsw/build_parallel.rs`

Action:

- defer broad annotation until Task 40 lifts the primitives,
- only patch immediate correctness bugs or resource leaks found during review.

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
