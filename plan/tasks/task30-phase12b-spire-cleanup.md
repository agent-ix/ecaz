# Task 30 Phase 12b: SPIRE Cleanup

Status: in progress
Owner: coder1 / SPIRE distributed production-hardening track
Priority: 1 before Phase 13 AWS verification adds new code on top of the
existing SPIRE surface; should ideally land before or alongside Phase
12a so 12a.1/12a.2/12a.6 do not pile into the same 12k-line file.

## Goal

Pay down the structural debt identified by the Phase 12 final review and
the RemoteScan/architecture audit (final-review packet `30982`,
architecture audit attached to the same conversation). Phase 12b is a
no-behavior-change refactor plus targeted test-coverage fills for the
`EcSpireDistributedScan` CustomScan entry points. The phase MUST NOT
introduce new SPIRE features and MUST NOT alter remote dispatch
semantics. Every change should be reviewable as a mechanical move plus
new tests.

## Entry State

- Phase 12 closed; tracker has zero unchecked rows.
- Phase 12a is complete. Final-review packet `30990` accepted closure
  at source SHA `46701d95`; Phase 12b now owns cleanup-only structural
  follow-up.
- Architecture audit measurements at this entry SHA:
  - `src/am/ec_spire/root/remote_candidates.rs` = 12,405 lines (kitchen
    sink: vocab, fault matrix, endpoint identity, payload decoder,
    dispatch, operator surface, two inline `#[cfg(test)]` blocks).
  - `src/lib.rs` = 66,054 lines (247 `test_ec_spire_*` PG18 fixtures in
    one translation unit).
  - `custom_scan.rs` = 2,946 lines (FFI thunks, plan-private, DML
    helpers, inline tests at 2621-2945).
  - Test layout split between inline `#[cfg(test)]`
    (`custom_scan.rs`, `cost.rs`, `dml_frontdoor.rs`, `options.rs`,
    `diagnostics.rs`, `vacuum.rs`, `assign.rs`, `quantizer.rs`) and
    external `tests.rs` / `tests/` (`scan/`, `build/`, `update/`,
    `meta/`, `storage/`, `root/`).
- Source SHA at task creation: branch `task-30-spire` HEAD `0b0f62f5`.
- First cleanup checkpoint source: branch `task-30-spire` after
  `7310eff9`, with Phase 12a final review recorded locally.

## Non-Goals

- New SPIRE features. Phase 12b is structural cleanup plus targeted
  test fills only.
- Behavior changes to remote dispatch, planner gating, CustomScan
  semantics, or operator-visible SQL. Pre-refactor and post-refactor
  test suites must pass without changes to assertions other than
  symbol path updates.
- Phase 12a P2 items. Those remain in `task30-phase12a-...`. This
  phase makes 12a easier to land cleanly.
- Rewriting unsafe FFI thunks. Move them, do not rewrite.

## Phase 12b.1: Split `root/remote_candidates.rs`

The file is a kitchen sink at 12,405 lines. Split it into a directory
module so 12a.1/12a.2/12a.6 can land without 3-way merge friction.

- [x] Convert `src/am/ec_spire/root/remote_candidates.rs` to
  `src/am/ec_spire/root/remote_candidates/mod.rs` with public re-exports
  preserving every external symbol path used elsewhere in the crate.
  First checkpoint preserves symbol paths with textual `include!`
  files in the existing `ec_spire` scope; no caller path changes were
  required beyond the top-level include.
- [x] Extract into sibling files:
  - [x] `remote_candidates/vocab.rs`: status-string constants
    (current lines 23-120) and hint strings.
  - [x] `remote_candidates/sort.rs`: candidate comparator
    (current lines 1-21).
  - [x] `remote_candidates/endpoint_identity.rs`: typed identity struct
    and validation (current ~8783-8900).
  - [x] `remote_candidates/payload.rs`:
    `decode_remote_search_typed_tuple_payload_pg_row` and tuple decode
    helpers (current ~9468-9581).
  - [x] `remote_candidates/fault_matrix.rs`: `production_fault_matrix_row`,
    matrix builder, and uniqueness assertion (current ~6300-6800,
    11555-11591).
  - [x] `remote_candidates/dispatch.rs`:
    `run_insert_prepare_requests_with_local_cancel_source`,
    `run_one_insert_prepare_request`, `selected_pids` plumbing
    (current ~3437-3582, 4154-4198).
  - [x] `remote_candidates/resolve.rs`:
    `coordinator_insert_resolve_remote_prepared` and prepared-xact
    cleanup helpers (current ~2230).
  - [x] `remote_candidates/operator.rs`: pg_extern surface including
    `ec_spire_remote_search_degraded_skip_report` (current ~12221) and
    other operator-callable wrappers.
  - [x] `remote_candidates/tests/` (subdirectory): move the two inline
    `#[cfg(test)] mod` blocks (current ~8842 and ~11437) into
    co-located files matching the split
    (`tests/endpoint_identity.rs`, `tests/payload.rs`,
    `tests/fault_matrix.rs`, etc.).
    Packets `30991` and `30993` moved `production_executor_state_tests`
    and the endpoint-identity tuple-transport tests.
- [ ] Verify zero behavior change: `cargo test -p ecaz` and the
  packet-local PG18 fixtures cited in Phase 11/12 trackers pass without
  assertion edits.
- [x] Verify the split with a per-file line-count sanity:
  no resulting file should exceed 2,500 lines; flag any that do as a
  follow-up split row.
- [x] Update any `pub(crate) use` paths in dependents
  (`custom_scan.rs`, `dml_frontdoor.rs`, `cost.rs`, `scan/`, build/
  callbacks).

## Phase 12b.2: Split `src/lib.rs` PG18 fixture sink

`src/lib.rs` at 66,054 lines is the single translation unit for 247
PG18 fixtures. Editor and `rustc` compile time are already painful;
Phase 13 will push it past 80k.

- Midphase audit decision in packet `31017`: the hard 2,500-line cap in
  the exit criteria applies to production files under `src/am/ec_spire/`,
  not to the temporary `src/tests/` fixture sink. `src/tests/` concern
  files should still be reported in line-count artifacts, and any
  obviously unreviewable concern file should get a follow-up split row,
  but Phase 12b.2 should finish fixture evacuation before opening
  second-order fixture sub-splits.
- Midphase audit decision in packet `31017`: the original
  `src/lib.rs <2,000 lines` target is revised. Phase 12b closure now
  requires `src/lib.rs` to contain no `test_ec_spire_*` fixture bodies
  and to retain only pgrx registration, re-exports, pg_extern wrappers,
  and the pg_test scaffold. Further lib.rs module extraction is a
  follow-up only if a later reviewer asks for it.

- [x] Create `src/tests/` module tree (or `src/sql_fixtures/` if a name
  more accurately reflects content). Subdirectories per concern:
  - [x] `tests/insert.rs`
  - [x] `tests/scan.rs`
  - [x] `tests/custom_scan.rs`
  - [x] `tests/remote_search.rs`
  - [x] `tests/dml_frontdoor.rs`
  - [x] `tests/placement.rs`
  - [x] `tests/vacuum.rs`
  - [x] `tests/cost_and_planner.rs`
  - [x] `tests/build.rs`
  - [x] `tests/diagnostics.rs`
- First fixture-sink checkpoint creates `src/tests/mod.rs` and moves the
  `#[pg_schema] mod tests` body out of `src/lib.rs`; concern-specific
  subfiles remain open.
- Packet `30998` moves the contiguous CustomScan pg_test fixture block to
  `src/tests/custom_scan.rs` with a textual include so fixture names and
  pg_schema scope remain unchanged; the other concern-specific files
  remain open.
- Packet `30999` starts `src/tests/remote_search.rs` by moving the first
  contiguous remote-search contract fixture block with a textual include;
  later tuple-payload, libpq, remote-node, and degraded-mode fixtures
  remain open, so the `tests/remote_search.rs` row is not yet closed.
- Packet `31026` moves the remaining reaper, remote PK isolation,
  tuple-payload, libpq, remote-node, manifest, and degraded-mode fixtures
  to `src/tests/remote_search.rs`; `src/tests/mod.rs` now retains no
  remote-search concern block, so the `tests/remote_search.rs` row is
  closed.
- Packet `31000` starts `src/tests/insert.rs` by moving the contiguous
  coordinator-insert and insert-trigger fixture block with a textual
  include; later insert-after-build fixtures remain open, so the
  `tests/insert.rs` row is not yet closed.
- Packet `31018` extends `src/tests/insert.rs` with the coordinator
  insert schema-drift and remote schema-fingerprint pre-dispatch
  fixtures; later insert-after-build and source-identity fixtures remain
  open, so the row is still not closed.
- Packet `31019` extends `src/tests/insert.rs` with the later
  post-build multi-row, post-build validation, and empty-index bootstrap
  fixture block; earlier insert-after-build delta, concurrent same-leaf,
  and source-identity fixtures remain open, so the row is still not
  closed.
- Packet `31020` extends `src/tests/insert.rs` with the earlier
  post-build delta, same-leaf delta, and PG18 concurrent same-leaf
  insert fixtures; source-identity fixtures remain open, so the row is
  still not closed.
- Packet `31021` moves the source-identity fixture block to
  `src/tests/insert.rs` and fixes the shared heap-resolution helper to
  allow one-key INCLUDE indexes; `src/tests/mod.rs` now retains no
  insert concern block, so the `tests/insert.rs` row is closed.
- Packet `31001` starts `src/tests/dml_frontdoor.rs` by moving the main
  DML hook/plan/remote-customscan fixture block with a textual include;
  earlier select-plan and later primitive-plan fixtures remain open, so
  the `tests/dml_frontdoor.rs` row is not yet closed.
- Packet `31004` extends `src/tests/dml_frontdoor.rs` with the later
  primitive-plan helper fixture block; earlier select-plan and
  replacement-decision SQL fixtures remain open, so the row is still not
  closed.
- Packet `31005` extends `src/tests/dml_frontdoor.rs` with the earlier
  PK-select/custom-scan plan fixtures and replacement-decision SQL
  fixture; broader coordinator update/delete/select tuple-payload SQL
  fixtures remain in `src/tests/mod.rs`, so the row is still not closed.
- Packet `31006` moves the remaining coordinator update/delete/select
  tuple-payload and update/delete schema-drift fixtures to
  `src/tests/dml_frontdoor.rs`; `src/tests/mod.rs` now only retains the
  DML concern include, so the `tests/dml_frontdoor.rs` row is closed.
- Packet `31007` starts `src/tests/diagnostics.rs` by moving the
  hierarchy/object/delta/options snapshot fixture block with a textual
  include; later scan-sanity, health, relation-storage, top-graph, and
  placement diagnostic fixtures remain open, so the row is not yet
  closed.
- Packet `31010` extends `src/tests/diagnostics.rs` with the scan-sanity,
  health, and relation-storage snapshot fixture block; later top-graph,
  active/allocator, and placement diagnostic fixtures remain open, so the
  row is still not closed.
- Packet `31011` extends `src/tests/diagnostics.rs` with the active
  snapshot, large-routing diagnostics, and allocator snapshot fixture
  block; later top-graph and placement diagnostic fixtures remain open,
  so the row is still not closed.
- Packet `31012` moves the remaining top-graph snapshot and boundary
  replica placement diagnostics fixtures to `src/tests/diagnostics.rs`;
  the leaf snapshot fixture remains open, so the row is still not closed.
- Packet `31013` moves the leaf snapshot fixture to
  `src/tests/diagnostics.rs`;
  `src/tests/mod.rs` now only retains the diagnostics concern include, so
  the `tests/diagnostics.rs` row is closed.
- Packet `31008` starts `src/tests/build.rs` by moving the initial
  boundary-replica, recursive boundary-replica, and PQ-FastScan populated
  build-deferral fixtures with a textual include; later populated-build,
  multistore, recursive-fanout, and top-graph build fixtures remain open,
  so the row is not yet closed.
- Packet `31014` extends `src/tests/build.rs` with the populated build
  root-control and logical-store hash-routing fixture block; later
  multistore, recursive-fanout, and top-graph build fixtures remain open,
  so the row is still not closed.
- Packet `31015` extends `src/tests/build.rs` with the auxiliary-store
  relcache, multistore, reindex, tqvector populated-build, and two-store
  scan fixture block; later recursive-fanout and top-graph build fixtures
  remain open, so the row is still not closed.
- Packet `31016` moves the remaining recursive-fanout and large top-graph
  chain-storage fixtures to `src/tests/build.rs`; `src/tests/mod.rs` now
  only retains the build concern include, so the `tests/build.rs` row is
  closed.
- Packet `31009` starts `src/tests/vacuum.rs` by moving the epoch cleanup,
  epoch snapshot, and maintenance-run fixture block with a textual
  include; later SQL VACUUM and concurrent insert/vacuum fixtures remain
  open, so the row is not yet closed.
- Packet `31022` moves the remaining delete-delta, cleanup compaction,
  SQL VACUUM, multistore SQL VACUUM, and concurrent insert/vacuum
  fixtures to `src/tests/vacuum.rs`; `src/tests/mod.rs` now retains no
  vacuum concern block, so the `tests/vacuum.rs` row is closed.
- Packet `31002` starts `src/tests/placement.rs` by moving the placement
  catalog and placement-snapshot fixture block with a textual include;
  scan-placement and later contention/diagnostic fixtures remain open, so
  the `tests/placement.rs` row is not yet closed.
- Packet `31023` moves the remaining placement write-contention fixture to
  `src/tests/placement.rs`; scan-placement lives in `src/tests/scan.rs`
  and placement diagnostics live in `src/tests/diagnostics.rs`, so the
  `tests/placement.rs` row is closed.
- Packet `31003` starts `src/tests/scan.rs` by moving the scan-placement,
  scan pipeline, routing, and centroid-classification fixture block with
  a textual include; later build/scan fixtures remain open, so the
  `tests/scan.rs` row is not yet closed.
- Packet `31024` moves the remaining empty ordered-scan and
  flat-vs-recursive scan parity fixtures to `src/tests/scan.rs`;
  `src/tests/mod.rs` now retains no scan concern block, so the
  `tests/scan.rs` row is closed.
- Packet `31025` creates `src/tests/cost_and_planner.rs` for the
  remaining SPIRE access-method, operator-class, and CustomScan
  planner/hook status registration fixtures; the
  `tests/cost_and_planner.rs` row is closed.
- Packet `31027` moves the remaining relation object/leaf storage and
  empty-manifest publish roundtrip fixtures into `src/tests/diagnostics.rs`;
  `src/tests/mod.rs` now retains shared helpers and concern-file includes,
  not direct `test_ec_spire_*` fixture bodies, so the test module-tree row
  is closed.
- [x] Move all `test_ec_spire_*` PG18 fixture functions out of
  `src/lib.rs` into the matching test file. Keep `lib.rs` for
  registration, re-exports, and the actual pgrx extension entry points.
  Packet `30995` moved the fixture bodies to `src/tests/mod.rs`;
  concern-specific files remain open under the preceding row.
- [x] Resolve the `src/lib.rs <2,000 lines` target after the midphase
  audit: the hard numeric target is replaced by the fixture-body absence
  requirement above, plus packet-local line-count evidence.
- [ ] Verify `cargo pgrx test` passes against PG18 with no fixture name
  changes. Coverage for cited tracker rows must remain at the same
  function names so the Phase 11/12/12a trackers do not require edits.
- [x] Spot-check 10 random tracker rows that cite a fixture name; each
  should still resolve via `rg test_ec_spire_<name> src/tests/`.
  Packet `31029` records the selected tracker strings and their
  `src/tests/` locations; all ten resolved.

## Phase 12b.3: Split `custom_scan.rs` and fill RemoteScan test gaps

Two-part: structural split, then add the missing Rust-level tests for
the FFI entry points that today have only shell-fixture coverage.

### Structural split

- [x] Convert `src/am/ec_spire/custom_scan.rs` (2,946 lines) to
  `src/am/ec_spire/custom_scan/mod.rs`.
- [x] Extract into sibling files:
  - [x] `custom_scan/plan_private.rs`: plan-mode and private decode
    helpers (current ~992-1170 region).
  - [x] `custom_scan/begin_exec.rs`: `ec_spire_begin_custom_scan`,
    `ec_spire_exec_custom_scan`, `ec_spire_end_custom_scan`,
    `ec_spire_rescan_custom_scan` FFI thunks
    (current 1673-1778 region) plus their helper state-mgmt functions.
  - [x] `custom_scan/dml.rs`: DML-specific CustomScan plan/exec
    helpers.
  - [x] `custom_scan/cost_helpers.rs`: cost-glue helpers that
    `cost.rs` calls into.
  - [x] `custom_scan/tests.rs`: relocate the inline tests block
    (current 2621-2945) to a sibling file.

### RemoteScan test fills (audit-identified gaps)

- [x] `BeginCustomScan` state-struct unit test: drive
  `ec_spire_begin_custom_scan` against a minimal in-memory plan and
  assert the resulting state contains the expected fanout descriptor
  count, planned output count, and zero progress counters. Covered at
  decoded plan-part level by the vector-order begin-state helper that
  `BeginCustomScan` invokes; the unit test avoids direct tuple descriptor
  and expression initialization outside a PostgreSQL backend.
- [x] `EndCustomScan` cleanup test: build a state and assert no
  Rust-level reachable allocations remain after cleanup (Miri-friendly
  shape if practical; otherwise a leak-counter test using `palloc`/`pfree`
  pairings counted by a test hook). Covered by the Rust-state release
  helper that `EndCustomScan` invokes; the unit test avoids direct
  `#[pg_guard]` thunk calls outside a PostgreSQL backend.
- [x] `ReScanCustomScan` test (**audit flagged this as a real gap**):
  - [x] Drive a CustomScan through full exhaustion via the production
    output-cursor helper;
  - [x] Call the production rescan reset helper that the FFI thunk
    invokes;
  - [x] Verify `outputs`, `next_output`, `loaded_outputs` are reset and
    that the second pass returns the same row set.
- [x] Read-path cancellation Rust test: today only the INSERT-side has
  one (`test_ec_spire_insert_prepare_local_cancel_rolls_back`). Add a
  symmetric read-path test that drives the CustomScan, sets local
  cancel, and asserts the executor unwinds with
  `local_query_cancelled` and no leaked transport state. The CustomScan
  pg_test asserts the PostgreSQL read query is interrupted at the
  backend boundary; the existing receive-layer local-cancel fixture
  covers `local_query_cancelled` categorization and governance lock
  release.
- [x] `ExplainCustomScan` contract: implement at least a minimal
  `ExplainCustomScan` callback that emits a stable JSON shape with
  `node = EcSpireDistributedScan`, `remote_fanout`,
  `tuple_transport_status`, `nprobe`, `rerank_width`. Add a fixture
  that runs `EXPLAIN (FORMAT JSON)` on a canonical query and asserts
  the shape. This replaces today's two shell `grep`s on plan text.
  Packet `30996` wires the callback and extends the loopback remote
  tuple-payload fixture with `EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF)`
  assertions for the stable shape.
- [x] Empty-remote-result CustomScan fixture: a remote that returns
  zero rows for a valid query; assert the CustomScan returns zero rows
  cleanly with no `not_applicable` status leakage. Packet `31028` adds
  a loopback CustomScan fixture that probes the remote endpoint returning
  zero rows, asserts the CustomScan plan stays active, and checks
  `EXPLAIN (FORMAT JSON, ANALYZE)` for ready tuple transport without
  `not_applicable`.

## Phase 12b.4: Standardize test layout

Pick one convention and migrate. Current mix is historical drift.

- [x] Decision: every module uses an external `tests.rs` (or `tests/`
  subdirectory if the test surface is large enough to need a split).
  Inline `#[cfg(test)] mod tests` is removed.
  Decision is recorded in `docs/SPIRE_CODE_LAYOUT.md`.
- [x] Migrate the inline-test files to the chosen convention:
  - [x] `cost.rs` → `cost/tests.rs`
  - [x] `dml_frontdoor.rs` → `dml_frontdoor/tests.rs`
  - [x] `options.rs` → `options/tests.rs`
  - [x] `diagnostics.rs` → `diagnostics/tests.rs`
  - [x] `vacuum.rs` → `vacuum/tests.rs`
  - [x] `assign.rs` → `assign/tests.rs`
  - [x] `quantizer.rs` → `quantizer/tests.rs`
  - [x] `custom_scan.rs` already covered by 12b.3.
- [x] Document the convention in `docs/SPIRE_DIAGNOSTICS.md` (or a new
  short `docs/SPIRE_CODE_LAYOUT.md`) so the next contributor does not
  reintroduce the drift.

## Phase 12b.5: Rename `root/` → `coordinator/`

`root/` is a misnomer. Originally it meant "root-node ops"; today it
hosts cross-cutting coordinator/fanout concerns (remote_candidates,
snapshots, hierarchy_snapshots, diagnostics, lifecycle, maintenance,
types, debug).

- [x] Rename `src/am/ec_spire/root/` to `src/am/ec_spire/coordinator/`.
- [x] Update every `use crate::am::ec_spire::root::...` path across the
  crate. Packet `30997` confirms there were no Rust `root::` module
  paths; the old folder was an include-only implementation layout.
- [x] Verify the rename with a `git grep root::` sanity pass; remaining
  matches should be unrelated (e.g. `tree.root`, `ec_spire_root_*` SQL
  identifiers — those stay).
- [x] Decide whether SQL/identifier `root` names (e.g.
  `ec_spire_root_control_state`) also need renaming. Default: **no**.
  Operator-visible identifiers are stable; only the Rust module name
  changes. Packet `30997` records the decision to leave `root/control`
  wording and SQL identifiers stable.

## Phase 12b.6: Reduce unsafe / business-logic mixing

Audit-flagged smell: `dml_frontdoor.rs` and `update/` carry `unsafe`
mixed with business logic.

- [x] Identify every `unsafe` block in `dml_frontdoor.rs` and `update/`
  (and any other non-FFI module).
- [x] For each block, classify: (a) truly FFI/SPI-boundary, move into
  an `ffi.rs` sibling; (b) avoidable, replace with safe abstraction.
- [x] If any block is genuinely required at the business-logic layer
  with no clean abstraction, leave with a one-line `// SAFETY:` comment
  naming the invariant.
- [x] Re-run the count: `unsafe`-bearing lines in `src/am/ec_spire/`
  should not increase; goal is a small reduction with concentration at
  the FFI surface.

## Phase 12b.7: Re-audit `unwrap()` / `expect()` density in non-test
paths

The earlier audit reported 1,565 total occurrences, most in tests.
Post-12b.1/12b.2/12b.3 splits, re-run with proper test exclusion.

- [x] `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire/ --glob '!**/tests*'`
  after the splits land.
- [x] For each non-test hit, classify as: (a) infallible by upstream
  guarantee, leave with a one-line comment naming the invariant;
  (b) replaceable with `?` or explicit error; (c) hot-path panic risk
  on remote-supplied data, fix or add a bounds check.
- [x] Category (c) hits must be zero before Phase 13. Record any
  accepted (a)-category exceptions in the packet evidence.

## Suggested Packet Sequence

These are mostly independent and can interleave with Phase 12a if
needed, but the `remote_candidates.rs` split should land first to
reduce 12a merge friction.

1. `12b.1` split `remote_candidates.rs` — biggest merge-friction
   reducer; do this before 12a.1/12a.2/12a.6 land.
2. `12b.3` `custom_scan.rs` split + RemoteScan test fills — the
   `ReScanCustomScan` gap and `ExplainCustomScan` contract are
   load-bearing for Phase 13 EXPLAIN-driven tuning.
3. `12b.2` `src/lib.rs` fixture split — large mechanical change,
   self-contained.
4. `12b.4` test layout standardization — fold into the above splits
   where convenient.
5. `12b.5` `root/` rename — late so other splits do not need to
   reflow paths twice.
6. `12b.6` unsafe consolidation — after splits, since FFI thunks are
   now in their own modules.
7. `12b.7` unwrap/expect non-test audit — last, since the
   test-exclusion glob depends on the post-split layout.

## Exit Criteria

- No file in `src/am/ec_spire/` exceeds 2,500 lines (modulo
  reviewer-accepted exceptions).
- `src/tests/` concern files are exempt from the production-file
  2,500-line hard cap during Phase 12b fixture evacuation; the final
  closeout records the largest fixture files and schedules follow-up
  sub-splits only if reviewability becomes a blocker.
- `src/lib.rs` contains no `test_ec_spire_*` fixture bodies. It may
  retain pgrx registration, re-exports, pg_extern wrappers, and the
  pg_test scaffold even when that keeps the file above 2,000 lines.
- Every CustomScan entry point (`Begin`, `Exec`, `End`, `ReScan`,
  `Explain`) has at least one Rust-level unit or fixture test asserting
  observable post-call state or output shape.
- `ExplainCustomScan` emits a stable JSON shape, asserted by a fixture.
- Test layout is consistent (external `tests.rs` / `tests/` across the
  crate); inline `#[cfg(test)] mod tests` is absent from non-test
  source files.
- `root/` has been renamed (or the rename has been explicitly deferred
  with reviewer-accepted rationale in the packet).
- No new `unsafe` blocks outside the FFI surface; non-test
  `unwrap()`/`expect()` category (c) hits are zero.
- `cargo test`, `cargo pgrx test`, and the packet-local PG18 fixtures
  pass without assertion edits other than symbol-path updates from the
  splits.
