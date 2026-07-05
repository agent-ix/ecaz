# Task 50 Comprehensive Unsafe Burndown Plan

This packet replaces the prior Task 50 execution ladder. The old plan optimized
for a top-15 metric and a small set of nine structural candidates. That is not
the current objective.

The new objective is broader and stricter:

> Remove every direct `unsafe { ... }` block that can reasonably be removed.
> Remaining unsafe must be irreducible FFI / PostgreSQL / CPU-intrinsic boundary
> code, centralized behind named contracts, and recorded in a residual registry.

There is no 30% success framing in this plan. A file is not done because it
crosses a percentage threshold. A file is done only when every unsafe in that
file has one of these dispositions:

1. deleted outright;
2. replaced by a safe typed API or existing guard;
3. absorbed into a shared boundary helper that removes caller unsafe at scale;
4. marked irreducible with a specific invariant, owner module, and validation.

## Current Inventory Basis

This plan was written against the current dirty working tree. The tree includes
an uncommitted partial heap-slot helper slice in:

- `src/am/common/heap_slot.rs`
- `src/am/common/mod.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_diskann/scan_state.rs`

Those edits should be treated as paused implementation work. They can become
the first implementation slice after this plan is accepted, but they are not
counted as completed work in this plan.

Packet-local inventory artifacts:

- `artifacts/src-unsafe-block-count-working-tree.log`
- `artifacts/src-unsafe-block-lines-working-tree.log`
- `artifacts/repo-unsafe-block-count-working-tree.log`
- `artifacts/subsystem-totals-working-tree.log`
- `artifacts/pattern-candidate-counts-working-tree.log`

Current `src/` direct unsafe total: `2446` blocks across `131` files.

Subsystem totals:

| Subsystem | Blocks | Files |
| --- | ---: | ---: |
| HNSW | 797 | 12 |
| SPIRE | 771 | 45 |
| IVF | 252 | 10 |
| DiskANN | 195 | 7 |
| Tests | 185 | 36 |
| AM common | 84 | 7 |
| Root / other | 57 | 4 |
| Quant | 55 | 2 |
| Storage guards | 50 | 8 |

Non-`src` unsafe is also in scope for disposition:

- `hardening/careful/src/spire.rs`: 59
- `hardening/careful/src/pg_guards.rs`: 35
- `crates/ecaz-cli/src/commands/dev/fault.rs`: 1
- `crates/ecaz-lints/fixtures/panic_across_ffi/src/lib.rs`: 1
- vendored `vendor/hnsw_rs`: separate decision; do not edit vendored code
  blindly without deciding whether this repo owns that fork.

## Strategic Method

Do not work file-by-file by count alone. Counts decide where payoff is high;
contracts decide the order.

The execution strategy is:

1. Build a durable unsafe ledger from `src-unsafe-block-lines-working-tree.log`.
   Every direct unsafe gets an ID, category, owner program, planned disposition,
   and status.
2. Land shared contracts before broad rollout. A shared contract is only worth
   adding when it deletes unsafe from multiple callers or makes future unsafe
   impossible through types.
3. Roll contracts through production-priority surfaces first:
   SPIRE and IVF/RaBitQ, then HNSW and DiskANN, then tests/debug/hardening.
4. Keep a residual registry. Any unsafe that remains must state why it cannot be
   removed and which boundary owns it.
5. Re-run the ledger after each packet. No packet closes until it updates
   before/after direct unsafe counts and the per-unsafe ledger statuses for all
   touched files.

## Unsafe Families And Contract Programs

The initial pattern inventory found recurring unsafe families. The plan is built
around these programs; each program has many rollout tranches.

### P0. Unsafe Ledger And Guardrails

Purpose: prevent blind cleanup.

Work:

- Generate `unsafe-ledger.jsonl` from `src-unsafe-block-lines-working-tree.log`.
- Fields: `id`, `file`, `line_at_capture`, `enclosing_item`, `category`,
  `program`, `disposition`, `status`, `residual_reason`, `packet`.
- Add a check that blocks new direct unsafe unless the ledger is updated.
- Define category vocabulary used by every packet.

Coverage: every `unsafe { ... }` row in the line-level artifact.

### P1. FFI And Callback Boundary Contracts

Pattern: AM callbacks, `extern "C-unwind"`, `pgrx_extern_c_guard`,
`pg_am_callback`, callback-private data.

Contract goal: all callback bodies delegate immediately into safe Rust entry
points. The only unsafe remains the ABI boundary and callback-private pointer
construction.

Rollout targets:

- AM routines: `src/am/ec_hnsw/routine.rs`, `src/am/ec_ivf/routine.rs`,
  `src/am/ec_spire/routine.rs`, `src/am/ec_diskann/routine.rs`.
- Callback-heavy files:
  `src/am/ec_hnsw/{scan.rs,vacuum.rs,build.rs,build_parallel.rs}`,
  `src/am/ec_ivf/{scan.rs,build.rs,vacuum.rs,cost.rs,options.rs}`,
  `src/am/ec_spire/{cost/mod.rs,vacuum/mod.rs,insert.rs,scan/callbacks.rs}`,
  `src/am/common/{callback.rs,cost.rs,explain.rs,parallel.rs,stream.rs}`.
- Root extension exports: `src/lib.rs`, `src/am/mod.rs`,
  `src/pg18_pgstat_shim.rs`, `src/standalone_pg_backend_stubs.rs`.

Expected disposition: most caller unsafe deleted; residual ABI unsafe centralized
and registered as irreducible.

### P2. PostgreSQL Handle Views

Pattern: repeated raw `pg_sys::Relation`, `IndexScanDesc`,
`IndexBuildHeapScan`, `IndexVacuumInfo`, `IndexBulkDeleteResult`, `ScanKey`,
`PlannerInfo`, `Query`, and `Node` pointer dereferences.

Contract goal: typed borrowed views with checked constructors at the callback
boundary, for example `LiveIndexRelation<'a>`, `IndexScanView<'a>`,
`BuildCallbackView<'a>`, `VacuumInfoView<'a>`, `ScanKeySlice<'a>`,
`PgNodeRef<'a, T>`.

Rollout targets:

- High-payoff scan/debug surfaces:
  `src/am/ec_hnsw/{scan.rs,scan_debug.rs,insert.rs,vacuum.rs,build.rs}`,
  `src/am/ec_ivf/scan.rs`,
  `src/am/ec_diskann/routine.rs`,
  `src/am/ec_spire/dml_frontdoor/mod.rs`.
- SPIRE planner/custom scan:
  `src/am/ec_spire/custom_scan/{planner.rs,plan_private.rs,dml.rs,begin_exec.rs,cost_helpers.rs,explain.rs,mod.rs,tests.rs}`.
- SPIRE coordinator/root relation readers:
  `src/am/ec_spire/coordinator/{snapshots.rs,hierarchy_snapshots.rs,debug.rs,maintenance.rs,diagnostics.rs,lifecycle.rs}` and remote-candidate modules.

Expected disposition: replace repeated `unsafe { (*ptr).field }` blocks with
safe methods on typed views.

### P3. Buffer, Page, And WAL Transaction Contracts

Pattern: `ReadBufferExtended`, lock/unlock/release, `BufferGetPage`,
`PageGet*`, `PageAddItem*`, free-space recording, `GenericXLogTxn`.

Contract goal: closure APIs that tie lock mode, page lifetime, WAL
registration, mutation, finish/abort, and free-space recording together.

Rollout targets:

- Shared guards: `src/storage/{buffer_guard.rs,wal.rs,relation_guard.rs,scan_guard.rs,snapshot_guard.rs,slot_guard.rs,lock_guard.rs,spi_guard.rs}`.
- IVF page/WAL: `src/am/ec_ivf/page.rs` plus callers in
  `build.rs`, `insert.rs`, `scan.rs`, `vacuum.rs`, `admin.rs`.
- SPIRE page/store: `src/am/ec_spire/page.rs`,
  `src/am/ec_spire/storage/{relation_store.rs,relation_plan.rs}`,
  `src/am/ec_spire/build/{drafts.rs,publish.rs,recursive.rs,tuples.rs}`.
- HNSW/DiskANN page mutation:
  `src/am/ec_hnsw/{insert.rs,vacuum.rs,shared.rs,graph.rs}`,
  `src/am/ec_diskann/{insert.rs,routine.rs,ambuild.rs,scan_state.rs}`.

Expected disposition: page/WAL unsafe remains in storage/page primitives only;
AM callers use safe read/write closures.

### P4. Page Tuple And Line-Pointer Views

Pattern: item-id bounds, tuple offsets, tuple lengths, `from_raw_parts`,
`PageGetItemId`, `PageGetItem`, mutable tuple bytes.

Contract goal: typed page tuple iterators and tuple reference wrappers. Immutable
views and mutable views must be separate so aliasing is enforced by the type
system.

Rollout targets:

- `src/am/ec_ivf/page.rs`: centroid, list directory, PQ codebook, posting tuple,
  RaBitQ posting payloads.
- `src/am/ec_spire/page.rs`: object tuple read/scan/rewrite/delete.
- `src/am/ec_hnsw/{shared.rs,graph.rs,insert.rs,vacuum.rs,scan.rs,scan_debug.rs}`.
- `src/am/ec_diskann/{insert.rs,routine.rs,ambuild.rs,scan_state.rs}`.

Expected disposition: callers never touch line pointers or raw tuple bytes
directly.

### P5. Heap Source, Tuple Slot, Snapshot, And Scorer Contracts

Pattern: heap relation fallback, snapshot fallback, reusable tuple slot,
`table_tuple_fetch_row_version`, `slot_getsomeattrs_int`, slot datum arrays,
heap-source scoring.

Contract goal: one cross-AM `HeapSourceScorer` / `HeapSlotReader` family that
owns relation guard, snapshot guard, slot lifetime, attribute resolution, and
scratch storage.

Rollout targets:

- First slice after plan acceptance can be the paused helper seed:
  `src/am/common/heap_slot.rs`,
  `src/am/ec_spire/scan/relation.rs`,
  `src/am/ec_diskann/scan_state.rs`.
- Then complete IVF/RaBitQ:
  `src/am/ec_ivf/scan.rs`, `insert.rs`, `build.rs`, `vacuum.rs`.
- Then SPIRE production read:
  `src/am/ec_spire/scan/relation.rs`,
  `src/am/ec_spire/storage/relation_store.rs`,
  `src/am/ec_spire/custom_scan/begin_exec.rs`.
- Then HNSW/DiskANN:
  `src/am/ec_hnsw/{source.rs,scan.rs,insert.rs,vacuum.rs,build.rs}`,
  `src/am/ec_diskann/{routine.rs,insert.rs,ambuild.rs}`.

Expected disposition: slot and heap-fetch unsafe removed from AM files; residual
unsafe lives in one common slot/heap module.

### P6. Datum, Varlena, Vector, And Quantized Payload Contracts

Pattern: `Datum`, `FromDatum`, detoast, varlena byte slices, ecvector/tqvector,
real arrays, source vector decoding, RaBitQ/PQ payload slices.

Contract goal: safe datum wrappers with scoped borrowed views and owned decode
paths where required:

- `EcVectorDatum<'a>`
- `TqVectorDatum<'a>`
- `FlatFloat4Source<'a>`
- `QuantizedPayloadRef<'a>`
- `OrderByQueryVector`

Rollout targets:

- Existing source wrappers:
  `src/am/ec_hnsw/source.rs`, `src/am/common/detoast.rs`.
- IVF/RaBitQ:
  `src/am/ec_ivf/{scan.rs,build.rs,insert.rs,quantizer.rs,page.rs}`.
- SPIRE:
  `src/am/ec_spire/{scan/relation.rs,custom_scan/dml.rs,custom_scan/tuple_payload.rs,dml_frontdoor/mod.rs,quantizer/mod.rs}`.
- DiskANN:
  `src/am/ec_diskann/{ambuild.rs,routine.rs,insert.rs}`.
- Quant kernels:
  `src/quant/{hadamard.rs,prod.rs}`.

Expected disposition: AM code stops detoasting and slicing datum bytes directly.

### P7. Reloptions And C String Contracts

Pattern: `rd_options`, reloption offset math, C-string conversion,
`format_type_be`, `CStr::from_ptr`, palloc string ownership.

Contract goal: per-AM typed reloption views and palloc string guards.

Rollout targets:

- `src/am/ec_hnsw/options.rs`
- `src/am/ec_ivf/options.rs`
- `src/am/ec_spire/options/mod.rs`
- `src/am/ec_diskann/options.rs`
- option consumers in AM cost/routine/admin files and SPIRE planner/DML code.

Expected disposition: no AM file decodes raw `rd_options` directly; C strings
are owned by a small guard.

### P8. DSM, Atomics, Shared Memory, And Lock Contracts

Pattern: DSM layout pointers, PostgreSQL atomics, LWLocks, shared counters,
parallel build slots.

Contract goal: typed shared-memory layouts where each field wrapper names its
memory-ordering and lock invariant.

Rollout targets:

- `src/am/ec_hnsw/build_parallel.rs`
- `src/am/common/{parallel.rs,parallel_slot.rs}`
- HNSW parallel consumers in `build.rs`, `scan.rs`, `scan_debug.rs`.
- Storage lock guard: `src/storage/lock_guard.rs`.

Expected disposition: DSM pointer arithmetic and atomic field access removed
from HNSW/common call sites.

### P9. Read Stream And Prefetch Contracts

Pattern: `read_stream_begin_relation`, callback-private state, per-buffer data,
`read_stream_next_buffer`, early return cleanup, `PrefetchBuffer`.

Contract goal: `ReadStreamGuard` and typed callback state wrappers that end the
stream on every exit path and expose typed per-buffer block metadata.

Rollout targets:

- `src/am/common/stream.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_spire/scan/relation.rs`
- `src/am/ec_spire/storage/relation_store.rs`
- HNSW/DiskANN prefetch paths in scan/vacuum/routine files.

Expected disposition: read-stream unsafe exists only inside the guard module.

### P10. Scan Opaque And Raw Ownership Contracts

Pattern: `Box::into_raw`, `Box::from_raw`, palloc scan opaque, `ptr::write`,
manual cleanup, scan-state raw member pointers.

Contract goal: typed scan/build/vacuum state owners with explicit allocate,
borrow, and free operations.

Rollout targets:

- `src/am/ec_hnsw/{scan.rs,scan_debug.rs,insert.rs,vacuum.rs,build.rs}`
- `src/am/ec_ivf/{scan.rs,insert.rs,build.rs,vacuum.rs}`
- `src/am/ec_diskann/routine.rs`
- `src/am/ec_spire/{scan.rs,custom_scan/begin_exec.rs,custom_scan/plan_private.rs}`
- root callback shims in `src/lib.rs`.

Expected disposition: each opaque pointer is converted once at the callback
boundary; internal helpers take references.

### P11. Planner, Node, List, And Custom Scan Views

Pattern: PostgreSQL node casts, planner hooks, List iteration, custom-scan
private data copy/read/write, SPIRE DML expression walkers.

Contract goal: safe AST/list wrappers and specific SPIRE DML plan views.

Rollout targets:

- `src/am/ec_spire/dml_frontdoor/mod.rs`
- `src/am/ec_spire/custom_scan/{planner.rs,plan_private.rs,dml.rs,begin_exec.rs,cost_helpers.rs,tuple_payload.rs,tests.rs}`
- `src/am/ec_spire/coordinator/remote_candidates/{scan_output.rs,write_payload.rs,dispatch.rs,fanout.rs,executor_receive.rs,libpq_plan.rs,operator.rs,pipeline.rs,result_contracts.rs,resolve.rs,endpoint_identity.rs,fault_matrix.rs}`

Expected disposition: node/list pointer unsafe consolidated into typed AST/list
iteration contracts.

### P12. SIMD, Quant, And Raw Memory Kernels

Pattern: target-feature functions, lane loads/stores, `from_raw_parts`,
`align_to`, `copy_nonoverlapping`, raw byte payloads.

Contract goal: architecture-specific lane wrapper types that own load/store
preconditions and scalar tail handling.

Rollout targets:

- `src/quant/hadamard.rs`
- `src/quant/prod.rs`
- RaBitQ/PQ consumers in IVF and SPIRE quantizer paths.

Expected disposition: unsafe remains only in the smallest architecture-specific
load/store constructors.

### P13. Tests, Debug Exports, Hardening, Crates, Vendor

Pattern: test-only `dlsym`, debug helper relation opens, remote-search fixtures,
hardening proof wrappers, lint fixtures, vendored code.

Contract goal: tests consume the same safe debug/export helpers as production;
test-only PG symbol mutation is isolated in one test support module.

Rollout targets:

- `src/tests/**`: 185 direct unsafe blocks across 36 files.
- `hardening/careful/src/{spire.rs,pg_guards.rs}`.
- `crates/ecaz-cli/src/commands/dev/fault.rs`.
- `crates/ecaz-lints/fixtures/panic_across_ffi/src/lib.rs`.
- `vendor/hnsw_rs/**`: classify as vendored. Either exclude formally, patch
  through a vendored-fork policy, or replace usage with wrapped APIs.

Expected disposition: test unsafe is not ignored; it is either deleted through
safe test helpers or recorded as test-only irreducible.

## Execution Order

This is intentionally more than nine slices. The work should run as a sequence
of contract programs and fanout tranches.

### Wave 0: Ledger And Baseline

1. Land the unsafe ledger tooling and generated initial ledger.
2. Mark every current unsafe as one of the programs P1-P13.
3. Add residual registry skeleton.
4. Add a "no new unledgered unsafe" check.

### Wave 1: Foundation Contracts

5. Callback entry contract rollout.
6. PostgreSQL handle view constructors.
7. Buffer/page/WAL closure APIs.
8. Page tuple view APIs.
9. Heap slot/source scorer seed.
10. Datum/vector wrapper seed.
11. Reloption/C-string guard seed.
12. Read-stream guard seed.
13. DSM atomic field seed.
14. Scan opaque owner seed.

### Wave 2: SPIRE And IVF/RaBitQ Production Fanout

15. SPIRE page/store tuple views.
16. SPIRE active epoch/read relation handle views.
17. SPIRE production read heap-source/scorer rollout.
18. SPIRE custom scan planner/node/list views.
19. SPIRE DML frontdoor expression walkers.
20. SPIRE remote-candidate coordinator views.
21. IVF page tuple and posting views to zero reducible page unsafe.
22. IVF scan/build/insert/vacuum heap-source and vector datum rollout.
23. IVF/RaBitQ quant payload wrappers.
24. IVF options/reloptions cleanup.

### Wave 3: HNSW And DiskANN Fanout

25. HNSW scan opaque/state owner rollout.
26. HNSW scan and scan-debug handle/page/scorer rollout.
27. HNSW graph/shared page tuple rollout.
28. HNSW insert/vacuum page/WAL rollout.
29. HNSW build/build_parallel DSM atomic rollout.
30. DiskANN routine callback/scan owner rollout.
31. DiskANN insert/routine page tuple and WAL rollout.
32. DiskANN ambuild/vector datum rollout.
33. DiskANN options/reloptions cleanup.

### Wave 4: Shared, Root, Quant, Tests, Hardening

34. Storage guard residual minimization.
35. AM common parallel/stream/explain/cost residual minimization.
36. Root exports and `src/am/mod.rs` safe facade cleanup.
37. SIMD lane newtypes and quant fanout.
38. Test support symbol/timeout helper.
39. Remote-search test helper rollout.
40. HNSW/IVF/SPIRE/DiskANN test unsafe cleanup.
41. Hardening/careful cleanup or explicit proof-harness residual registry.
42. Crate fixture disposition.
43. Vendor disposition decision.

### Wave 5: Residual Burnoff

44. Re-run full ledger and generate zero-reducible report.
45. For every remaining unsafe, require residual registry row:
    - owning module;
    - exact invariant;
    - why safe Rust cannot express it;
    - proof/validation;
    - whether future Rust/pgrx/PostgreSQL APIs could remove it.
46. Final pass over files with 1-5 blocks; delete small stragglers.
47. Final pass over tests/debug/hardening.
48. Closeout only when every unsafe row is either removed or registered as
    irreducible.

## File Coverage Rule

Every file in `artifacts/src-unsafe-block-count-working-tree.log` is covered by
one or more programs above. The file is not allowed to disappear from tracking
until its ledger rows are removed or residual-registered.

Primary file groups:

- HNSW: all `src/am/ec_hnsw/*.rs` files in the count artifact.
- SPIRE: all `src/am/ec_spire/**` files in the count artifact.
- IVF: all `src/am/ec_ivf/*.rs` files in the count artifact.
- DiskANN: all `src/am/ec_diskann/*.rs` files in the count artifact.
- Shared AM/storage/quant/root: all counted files under `src/am/common/`,
  `src/storage/`, `src/quant/`, `src/lib.rs`, `src/am/mod.rs`,
  `src/pg18_pgstat_shim.rs`, `src/standalone_pg_backend_stubs.rs`.
- Tests: all counted files under `src/tests/**`.
- Non-`src`: all counted files in `repo-unsafe-block-count-working-tree.log`
  receive an explicit owned/hardening/crate/vendor disposition.

## Packet Acceptance Rules

Each implementation packet must include:

- before/after direct unsafe counts for every touched file;
- ledger diff for every touched unsafe ID;
- the contract program and wave/tranche number it advances;
- explanation of any unsafe moved into a helper and proof that call-site unsafe
  was deleted at scale;
- focused compile/lint validation appropriate to the touched modules;
- focused runtime tests only when callback ordering, tuple visibility, scan
  ordering, WAL mutation, or vector decoding could drift.

Full benchmark matrices are not part of this plan unless a packet changes
candidate ordering, scoring math, payload bytes, WAL order, or allocation shape
on a hot path. When that happens, use the narrowest `ecaz bench suite` evidence
needed for that behavioral risk.

## Closeout Gate

Task 50 closes only when:

- no unledgered direct unsafe exists under the chosen scope;
- every original unsafe ledger row is removed or residual-registered;
- every helper introduced by this task has call-site deletion evidence;
- every residual unsafe has a named owner and invariant;
- the final packet reports counts for `src/`, hardening/crates, tests, and
  vendor disposition separately.

