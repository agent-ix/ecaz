# Task 50 Slice Sequence

## Priority Rule

Task 50 should prepare the surfaces needed for optimization profiling. The
priority order is:

1. Shared RaBitQ/quant kernels used by IVF and SPIRE.
2. IVF with RaBitQ storage/scoring enabled.
3. SPIRE production read path, the ultimate production target, after it can
   consume already-validated RaBitQ/IVF helpers.
4. Cross-AM helper rollout to HNSW and DiskANN, then anything else in the
   top-15.

HNSW and DiskANN are still part of the top-15 unsafe-density requirement, but
they should not consume the earliest slices unless they prove a helper needed
by SPIRE/IVF/RaBitQ.

## Pre-Slice 0: Baseline Snapshot

Before code changes:

- run direct unsafe-block distribution and store it under
  the Packet 002 tooling packet;
- capture the local fast-iteration baseline described in
  `bench-baseline-plan.md` / `local-bench-plan.md`;
- keep existing AWS baselines available for closeout confirmation;
- decide whether Task 30 phase 13d SPIRE evidence is durable enough under
  NFR-007 or capture a new SPIRE-specific AWS closeout baseline before Slice 3c
  closes.

Do not use `scripts/unsafe_comment_baseline.txt`; Task 35 made that file empty.
The gate metric is per-file direct `unsafe { ... }` block count. Callsite count
is supporting narrative only.

## Packet Ladder

1. Packet 002: land `make unsafe-block-count` or
   `scripts/unsafe_block_count.sh` with the `rg`/`grep` fallback and a fresh
   direct-count snapshot.
2. Packet 003: generate any missing local corpus fixtures, load the full
   profile/AM baseline spread, and capture
   `benchmarks/task-50-local-baseline/` for fast local iteration.
3. Packet 004: capture `benchmarks/task-50-spire-baseline/` only if Task 30
   phase 13d evidence is not durable enough for Slice 3c closeout.
4. Packet 005: Slice 1a, callback helper plus one IVF user.
5. Packet 006: Slice 1b, IVF callback rollout.
6. Packet 007: Slice 1c, SPIRE callback rollout.
7. Later packets: Slice 2 IVF page visitor, Slice 3a/3b/3c SPIRE anchor, then
   Slice 4 scorer and top-15 HNSW/DiskANN coverage-map follow-through.

## Slice 1a: Callback Wrapper Helper Seed

Target:

- helper in `src/am/common` or a narrowly named callback module;
- one smallest viable IVF callback user, preferably a low-risk cost/tree-height
  style callback before scan/build/vacuum rollout.

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
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`;
- no runtime bench required unless the first user is on a hot callback path.

Coordination:

- consumes `pgrx_extern_c_guard` directly;
- does not duplicate Task 41 relation/buffer/snapshot guards.

Metric:

- gate: direct unsafe-block reduction in touched files;
- narrative: callsites absorbed by the helper.

## Slice 1b: IVF Callback Rollout

Target:

- remaining IVF callback users in
  `src/am/ec_ivf/{cost,build,scan,vacuum,insert,options}.rs`;
- prioritize callbacks needed by IVF/RaBitQ profiling.

Measurement:

- before/after block count for each touched IVF file;
- compile/lint as in Slice 1a;
- local same-host comparison only if a hot callback shape changes in a way that
  could inhibit inlining; AWS confirmation waits for closeout.

Coordination:

- consumes the Slice 1a helper;
- does not change page visitors, reloptions, or heap scorer code in this slice.

## Slice 1c: SPIRE Callback Rollout

Target:

- SPIRE callback users in `src/am/ec_spire/{scan/callbacks,cost,insert,vacuum}.rs`;
- defer DML/frontdoor callback-like surfaces unless required for compile shape.

Measurement:

- before/after block count for each touched SPIRE file;
- compile/lint as in Slice 1a;
- local SPIRE smoke if a read-efficiency callback is touched;
- no SPIRE closeout performance claim until the AWS SPIRE baseline gap is
  closed.

Coordination:

- consumes the Slice 1a helper;
- cross-check active Task 30 phase 13d branches before touching
  `coordinator/snapshots.rs` or read-efficiency surfaces.

Current callback metric:

- direct `pgrx_extern_c_guard` matches are supporting callsite inventory;
- packet success is measured by direct unsafe-block count before/after.

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
- local IVF RaBitQ recall + QPS comparison for fast iteration;
- targeted runtime tests only if visitor semantics alter error paths or tuple
  selection order.

Coordination:

- consume Task 41 page/relation helpers such as `AccessShareIndexRelation`
  where applicable;
- do not create a parallel PG resource guard layer.

Expected result:

- `src/am/ec_ivf/page.rs` starts at 134 unsafe blocks. The slice should aim for
  at least 30% reduction in that file or explicitly explain the ceiling.

## Slice 3a: SPIRE Active Epoch Anchor Seed

Target:

- define the typed `ActiveEpochAnchor` or equivalent context;
- wire exactly one smallest production user, preferably a scan/relation helper
  that already proves the active-epoch chain.

Design:

- typed object proves `index_relation -> root_control -> active_epoch ->
  manifests -> placement_directory -> local_store_config`;
- expose owned snapshots or lifetime-scoped borrows only;
- avoid hiding relation lock decisions inside the anchor unless the guard
  already exists.

Measurement:

- before/after counts for touched SPIRE files;
- compile/lint;
- no broad read-efficiency claim in 3a.

Coordination:

- must not step on Task 30 phase 13d edits;
- check active branches before touching SPIRE coordinator files;
- consumes existing relation guards rather than opening relations itself.

## Slice 3b: SPIRE Snapshot Rollout

Target:

- `src/am/ec_spire/coordinator/snapshots.rs`;
- `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`;
- adjacent read-only snapshot helpers only where the anchor type naturally
  removes repeated active-epoch proof blocks.

Measurement:

- before/after counts for each touched SPIRE file;
- compile/lint;
- targeted tests if snapshot row shape or diagnostics can drift.

Coordination:

- keep lock decisions explicit in the caller or existing guard;
- do not fold publish/vacuum mutation paths into the snapshot rollout.

## Slice 3c: SPIRE Read-Efficiency Rollout

Target:

- production read-efficiency path closest to Task 30 phase 13d;
- `ec_spire_remote_search_production_read_profile` surface where the anchor
  removes repeated active-epoch/placement validation.

Measurement:

- before/after counts for touched SPIRE files;
- local SPIRE read-efficiency smoke during iteration;
- AWS closeout comparison against Task 30 phase 13d evidence or the new Task 50
  SPIRE baseline packet;
- no DML-frontdoor change in this slice unless required by compile shape.

Coordination:

- cross-check active Task 30 phase 13d branch before editing read profile code;
- do not include DML/frontdoor or publish-path restructuring in the same packet.

Expected result across 3a-3c:

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

- local IVF RaBitQ recall/QPS and SPIRE read-efficiency benches;
- AWS confirmation when closing out the touched hot path;
- block counts in touched files;
- tests for cache-hit semantics or error message ordering if changed.

Coordination:

- consume existing `HeapRelationGuard` / `IndexRelationGuard`;
- do not duplicate Task 41 PG resource RAII;
- avoid Task 40 concurrency-state lifts.

## Slice 5: Reloptions Or Vector Datum Wrapper

Pick based on what Task 39/47 lands:

- choose reloptions if recall/cost gates expose storage-format option parsing
  as a blocker for RaBitQ/SPIRE;
- choose vector datum wrapper if IVF/RaBitQ build or insert profiling is next.

Both should remain narrow and should not be mixed with page visitor or scorer
changes.

Coordination:

- reloptions work consumes existing AM option builders and should not change
  planner semantics;
- vector datum work consumes existing detoast helpers and should not duplicate
  Task 41 memory/lifetime wrappers.
