# Task 50 Slice Sequence

## Priority Rule

Task 50 should prepare the surfaces needed for optimization profiling. The
priority order is:

1. SPIRE production read path.
2. IVF with RaBitQ storage/scoring enabled.
3. Shared RaBitQ/quant kernels used by IVF and SPIRE.
4. Cross-AM helper rollout to HNSW and DiskANN.

HNSW and DiskANN are still part of the top-15 unsafe-density requirement, but
they should not consume the earliest slices unless they prove a helper needed
by SPIRE/IVF/RaBitQ.

## Pre-Slice 0: Baseline Snapshot

Before code changes:

- run direct unsafe-block distribution and store it under
  `benchmarks/task-50-baseline/artifacts/unsafe-block-count-baseline.log`;
- capture bench baselines listed in `bench-baseline-plan.md`;
- record exact HEAD SHA, corpus, fixture, storage format, and isolated/shared
  table choice in `benchmarks/task-50-baseline/manifest.md`.

Do not use `scripts/unsafe_comment_baseline.txt`; Task 35 made that file empty.

## Slice 1: Callback Wrapper Helper

Target:

- helper in `src/am/common` or a narrowly named callback module;
- first applications in `src/am/ec_ivf/{cost,build,scan,vacuum,insert}.rs`,
  `src/am/ec_spire/{scan/callbacks,cost,insert,vacuum}.rs`, and common
  callback helpers that affect RaBitQ/IVF or SPIRE.

Design:

- support closure callbacks: `am_callback(|| { ... })`;
- support function-pointer callbacks that currently call
  `pgrx_extern_c_guard(fn_name)`;
- keep helper `#[inline]`;
- centralize the callback-duration pointer invariant once.

Measurement:

- before/after block count for each touched file;
- `cargo fmt --all`;
- `cargo check --all-targets --no-default-features --features pg18,bench`;
- clippy target from Task 50 validation if the slice touches enough production
  code to justify it;
- no runtime bench needed unless the callback helper changes hot closure
  capture or inlining behavior.

Expected first target count:

- IVF callback guard sites: 20 current `pgrx_extern_c_guard` matches.
- SPIRE callback guard sites: 20 current matches in sampled production files.
- Shared/common callback sites: 13 current matches.

## Slice 2: IVF/RaBitQ Page Tuple Visitor

Target:

- `src/am/ec_ivf/page.rs`;
- follow-on application in `src/am/ec_ivf/scan.rs` and build/insert/vacuum
  paths that read or mutate posting/list/centroid tuples.

Design:

- typed tuple views over a locked buffer;
- helper owns the line-pointer count, item-id bounds, tuple offset, tuple
  length, and page-size validation;
- mutation helper must stay separate from immutable visitor;
- do not combine this with WAL transaction changes.

Measurement:

- before/after counts for `page.rs` and any touched caller;
- IVF RaBitQ recall + QPS baseline comparison;
- targeted runtime tests only if visitor semantics alter error paths or tuple
  selection order.

Expected result:

- `src/am/ec_ivf/page.rs` starts at 134 unsafe blocks. The slice should aim for
  at least 30% reduction in that file or explicitly explain the ceiling.

## Slice 3: SPIRE Active Epoch Anchor

Target:

- start in the production read/coordination path rather than DML:
  `src/am/ec_spire/scan/relation.rs`,
  `src/am/ec_spire/coordinator/snapshots.rs`,
  `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`, or a new shared module
  consumed by them.

Design:

- typed object proves `index_relation -> root_control -> active_epoch ->
  manifests -> placement_directory -> local_store_config`;
- expose owned snapshots or lifetime-scoped borrows only;
- avoid hiding relation lock decisions inside the anchor unless the guard
  already exists.

Measurement:

- before/after counts for touched SPIRE files;
- SPIRE Task 30 phase 13d read-efficiency baseline comparison;
- no DML-frontdoor change in this slice unless required by compile shape.

Expected result:

- reduce repeated active-epoch load/validation unsafe blocks in one SPIRE
  production read path while making the read-efficiency profiling surface safer.

## Slice 4: Heap Source Scorer Helper

Target:

- first applications in IVF scan rerank/insert/vacuum and SPIRE heap-rerank
  relation fallback;
- HNSW and DiskANN consume the helper after the priority surfaces validate
  allocation and copy behavior.

Design:

- object owns heap relation guard, active snapshot, reusable slot, source
  attribute resolution, and scratch buffers;
- scoring method returns borrowed or copied vector data with explicit slot
  reuse boundaries;
- avoid new per-candidate allocation.

Measurement:

- IVF RaBitQ recall/QPS and SPIRE read-efficiency benches;
- block counts in touched files;
- tests for cache-hit semantics or error message ordering if changed.

## Slice 5: Reloptions Or Vector Datum Wrapper

Pick based on what Task 39/47 lands:

- choose reloptions if recall/cost gates expose storage-format option parsing
  as a blocker for RaBitQ/SPIRE;
- choose vector datum wrapper if IVF/RaBitQ build or insert profiling is next.

Both should remain narrow and should not be mixed with page visitor or scorer
changes.
