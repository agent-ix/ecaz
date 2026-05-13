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

- [ ] Create `src/tests/` module tree (or `src/sql_fixtures/` if a name
  more accurately reflects content). Subdirectories per concern:
  - [ ] `tests/insert.rs`
  - [ ] `tests/scan.rs`
  - [x] `tests/custom_scan.rs`
  - [ ] `tests/remote_search.rs`
  - [ ] `tests/dml_frontdoor.rs`
  - [ ] `tests/placement.rs`
  - [ ] `tests/vacuum.rs`
  - [ ] `tests/cost_and_planner.rs`
  - [ ] `tests/build.rs`
  - [ ] `tests/diagnostics.rs`
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
- Packet `31000` starts `src/tests/insert.rs` by moving the contiguous
  coordinator-insert and insert-trigger fixture block with a textual
  include; later insert-after-build fixtures remain open, so the
  `tests/insert.rs` row is not yet closed.
- Packet `31001` starts `src/tests/dml_frontdoor.rs` by moving the main
  DML hook/plan/remote-customscan fixture block with a textual include;
  earlier select-plan and later primitive-plan fixtures remain open, so
  the `tests/dml_frontdoor.rs` row is not yet closed.
- [x] Move all `test_ec_spire_*` PG18 fixture functions out of
  `src/lib.rs` into the matching test file. Keep `lib.rs` for
  registration, re-exports, and the actual pgrx extension entry points.
  Packet `30995` moved the fixture bodies to `src/tests/mod.rs`;
  concern-specific files remain open under the preceding row.
- [ ] After the move, `src/lib.rs` should be under 2,000 lines.
- [ ] Verify `cargo pgrx test` passes against PG18 with no fixture name
  changes. Coverage for cited tracker rows must remain at the same
  function names so the Phase 11/12/12a trackers do not require edits.
- [ ] Spot-check 10 random tracker rows that cite a fixture name; each
  should still resolve via `rg test_ec_spire_<name> src/tests/`.

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

- [ ] `BeginCustomScan` state-struct unit test: drive
  `ec_spire_begin_custom_scan` against a minimal in-memory plan and
  assert the resulting state contains the expected fanout descriptor
  count, planned output count, and zero progress counters.
- [ ] `EndCustomScan` cleanup test: build a state, call End, assert no
  Rust-level reachable allocations remain (Miri-friendly shape if
  practical; otherwise a leak-counter test using `palloc`/`pfree`
  pairings counted by a test hook).
- [ ] `ReScanCustomScan` test (**audit flagged this as a real gap**):
  - [ ] Drive a CustomScan through full exhaustion;
  - [ ] Call rescan;
  - [ ] Verify `outputs`, `next_output`, `loaded_outputs` are reset and
    that the second pass returns the same row set.
- [ ] Read-path cancellation Rust test: today only the INSERT-side has
  one (`test_ec_spire_insert_prepare_local_cancel_rolls_back`). Add a
  symmetric read-path test that drives the CustomScan, sets local
  cancel, and asserts the executor unwinds with
  `local_query_cancelled` and no leaked transport state.
- [x] `ExplainCustomScan` contract: implement at least a minimal
  `ExplainCustomScan` callback that emits a stable JSON shape with
  `node = EcSpireDistributedScan`, `remote_fanout`,
  `tuple_transport_status`, `nprobe`, `rerank_width`. Add a fixture
  that runs `EXPLAIN (FORMAT JSON)` on a canonical query and asserts
  the shape. This replaces today's two shell `grep`s on plan text.
  Packet `30996` wires the callback and extends the loopback remote
  tuple-payload fixture with `EXPLAIN (FORMAT JSON, ANALYZE, COSTS OFF)`
  assertions for the stable shape.
- [ ] Empty-remote-result CustomScan fixture: a remote that returns
  zero rows for a valid query; assert the CustomScan returns zero rows
  cleanly with no `not_applicable` status leakage.

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

- [ ] Identify every `unsafe` block in `dml_frontdoor.rs` and `update/`
  (and any other non-FFI module).
- [ ] For each block, classify: (a) truly FFI/SPI-boundary, move into
  an `ffi.rs` sibling; (b) avoidable, replace with safe abstraction.
- [ ] If any block is genuinely required at the business-logic layer
  with no clean abstraction, leave with a one-line `// SAFETY:` comment
  naming the invariant.
- [ ] Re-run the count: `unsafe`-bearing lines in `src/am/ec_spire/`
  should not increase; goal is a small reduction with concentration at
  the FFI surface.

## Phase 12b.7: Re-audit `unwrap()` / `expect()` density in non-test
paths

The earlier audit reported 1,565 total occurrences, most in tests.
Post-12b.1/12b.2/12b.3 splits, re-run with proper test exclusion.

- [ ] `rg -n '\.unwrap\(\)|\.expect\(' src/am/ec_spire/ --glob '!**/tests*'`
  after the splits land.
- [ ] For each non-test hit, classify as: (a) infallible by upstream
  guarantee, leave with a one-line comment naming the invariant;
  (b) replaceable with `?` or explicit error; (c) hot-path panic risk
  on remote-supplied data, fix or add a bounds check.
- [ ] Category (c) hits must be zero before Phase 13. Record any
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
- `src/lib.rs` is under 2,000 lines and contains no
  `test_ec_spire_*_sql` fixture bodies.
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
