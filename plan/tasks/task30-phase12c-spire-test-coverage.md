# Task 30 Phase 12c: SPIRE Test Coverage

Status: planned
Owner: coder1 / SPIRE distributed production-hardening track
Priority: 1 before Phase 13 AWS verification opens new fault surface;
should land after Phase 12b cleanup and before Phase 13 entry.

## Goal

Close the SPIRE test-coverage gaps identified by the Phase 12c
coverage audit (review packet `31070`). Phase 12c is **test-only**:
no production code changes outside the minimum needed to make a path
testable (e.g. exposing a `#[cfg(test)]` constructor, or replacing
a `#[should_panic]` scaffold with the real implementation that
unblocks a positive assertion). All net code change must land in
`src/tests/`, `src/am/ec_spire/**/tests/`, or `#[cfg(test)]` blocks
within the production crate.

This phase exists because the Phase 12b cleanup made the code base
amenable to honest coverage measurement, and the resulting audit
found:

- FFI lifecycle paths (`CreateCustomScanState`, `EndCustomScan`,
  `ReScanCustomScan`, `recheck`) only indirectly exercised.
- Stage E fault matrix only ~half-live; the new 12a categories
  (`payload_too_large`, `tuple_transport_retired`) and the 6 Stage E
  lifecycle rows have no live coverage.
- Several marquee tests use loose assertions admitting real
  regressions (substring EXPLAIN checks, `>` cost comparisons,
  isolation matrix missing READ COMMITTED, idempotent-DELETE shape
  unpinned).
- Read-path schema drift, recall, and multi-remote fanout >1 are
  under-tested.

## Entry State

- Phase 12 closed; Phase 12a closed (`30982`/`30990`); Phase 12b
  closed (`31060`). Tracker trio has zero unchecked rows.
- Coverage audit at SHA `096609df` enumerated 50 gaps across 11 axes;
  see `review/31070-spire-phase12c-coverage-audit/` for the full
  per-axis findings.
- Production-side cleanup means coverage gaps are now diagnosable —
  151 cohesive files, RemoteScan FFI in its own directory, fault
  matrix and lifecycle matrix enumerated in a single place
  (`production_summary.rs:234,290`).

## Non-Goals

- Production code changes beyond minimum testability hooks. New
  behaviour belongs in Phase 13 or a follow-up phase, not here.
- AWS/RDS verification. Still Phase 13.
- Re-running Phase 11/12/12a/12b fixtures. Those gates stand.
- Performance benchmarking. Phase 13 owns AWS-scale perf measurement;
  this phase is correctness/coverage only.
- HNSW test coverage. The `ec_hnsw_*` test files are out of scope.

## Phase 12c.1: CustomScan FFI Lifecycle Coverage (P1)

Source: audit Axis A. The CustomScan callbacks at
`src/am/ec_spire/custom_scan/mod.rs:97-113` are mostly only exercised
indirectly via the loopback marquee fixture.

### 12c.1.a: Cursor-rescan fixture

Drives `ec_spire_rescan_custom_scan` (`begin_exec.rs:183`)
end-to-end — the helper unit test at `custom_scan/tests.rs:316` does
not.

- [x] Open cursor over CustomScan, fetch N/2 rows. Evidence:
  `test_ec_spire_customscan_cursor_move_first_rescans_sql`.
- [x] Issue `MOVE FIRST`, fetch all remaining rows. Evidence:
  `test_ec_spire_customscan_cursor_move_first_rescans_sql`.
- [x] Assert second-pass row set equals first-pass row set. Evidence:
  `test_ec_spire_customscan_cursor_move_first_rescans_sql`.
- [x] Assert `outputs` / `next_output` / `loaded_outputs` state
  fields are reset (instrument via diagnostic snapshot or
  `#[cfg(test)]` getter). Evidence:
  `custom_scan_rescan_snapshot_for_test` in
  `test_ec_spire_customscan_cursor_move_first_rescans_sql`.

### 12c.1.b: `EndCustomScan` palloc/pfree pairing fixture

Exercises `begin_exec.rs:170` post-cancel cleanup path.

- [ ] Capture `MemoryContextStats` baseline before scan.
- [x] Cancel mid-`ExecCustomScan` (interrupt). Evidence:
  `test_ec_spire_customscan_read_cancel_releases_transport`.
- [x] Assert `EndCustomScan` invoked exactly once on the cancel
  unwind path. Evidence:
  `test_ec_spire_customscan_read_cancel_releases_transport`.
- [ ] Assert `MemoryContextStats` returns to baseline after end.

### 12c.1.c: `recheck` callback pin test

Assert `ec_spire_custom_scan_recheck` (`begin_exec.rs:332`) returns
`true` unconditionally, documenting the stale-row contract at
`begin_exec.rs:332-338`. A regression to `false` would silently drop
rows during EvalPlanQual rerun.

- [x] Unit test: directly invoke the recheck callback with a
  synthetic state and assert `true`.
- [x] Code-comment cross-reference: link the test to the contract
  comment in `begin_exec.rs:332-338`.

### 12c.1.d: `MarkPos` / `RestrPos` planner-exclusion test

Today the callbacks are `None` (`mod.rs:106-107`) but no test pins
the planner-side exclusion.

- [x] Assert planner refuses a `MergeAppend` plan over CustomScan.
  Evidence: `test_ec_spire_customscan_not_below_mergeappend_sql`.
- [x] Assert planner refuses an inner-rescan nested-loop above
  CustomScan (where MarkPos/RestrPos would be required). Evidence:
  `test_ec_spire_customscan_not_inner_rescan_nested_loop_sql`.

### 12c.1.e: `BeginCustomScan` UPDATE-branch panic recovery

`dml_update_value_exprs_from_plan` (`begin_exec.rs:90-92`) is the
UPDATE branch.

- [x] Drive UPDATE branch with invalid column metadata. Evidence:
  `custom_scan_dml_update_metadata_error_releases_half_initialized_state`.
- [x] Assert panic during `Begin` does not leak half-initialized
  `SpireCustomScanExecState`. Evidence:
  `custom_scan_dml_update_metadata_error_releases_half_initialized_state`.

### 12c.1.f: `BeginCustomScan` DELETE-branch panic recovery

- [x] Drive DELETE branch with invalid column metadata. Evidence:
  `custom_scan_dml_delete_metadata_error_releases_half_initialized_state`.
- [x] Assert panic does not leak half-initialized state. Evidence:
  `custom_scan_dml_delete_metadata_error_releases_half_initialized_state`.

### 12c.1.g: Retire `#[should_panic]` scaffolds

`custom_scan.rs:777,838` use
`#[should_panic(expected="EcSpireDistributedScan production executor
blocked")]`. They prove planner+begin reach a placeholder, not
positive behaviour.

- [x] `custom_scan.rs:777` — replace with positive assertion now
  that the production executor is wired. If still required,
  document why and tighten the panic string to a versioned
  identifier. Evidence: `test_ec_spire_customscan_exec_returns_remote_tuple_payload_sql`.
- [x] `custom_scan.rs:838` — same treatment for the
  parameter-query variant. Evidence:
  `test_ec_spire_customscan_exec_accepts_parameter_query_sql`.

## Phase 12c.2: Stage E Fault Matrix — Live Coverage (P1)

Source: audit Axis C. The matrix at
`src/tests/remote_search/production_summary.rs:234` enumerates 11+
fault categories; only ~half have live fixtures. The 12a-era
categories have no live coverage at all.

### 12c.2.a: `payload_too_large` (12a.2)

- [x] Strict mode: encoder emits a payload exceeding
  `ec_spire.max_remote_payload_bytes_per_row`; assert
  `SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE`. Evidence:
  `test_ec_spire_customscan_large_text_projection_cap_sql` from
  packet `731`, accepted in reviewer feedback `31120`.
- [x] Degraded mode: same payload; assert
  `degraded_skipped_dispatch_count` increments and the matrix-row
  hint is surfaced. Evidence:
  `degraded_skip_report_hints_remote_payload_cap`.
- [x] Per-batch cap: payload count exceeds
  `ec_spire.max_remote_payload_rows_per_batch`; assert the same
  category fires before per-row allocation. Evidence:
  `production_receive_adapters_reject_selected_pid_batches_before_connection`
  accepted in reviewer feedback `31090`.

### 12c.2.b: `tuple_transport_retired` (12a.5)

Stub remote advertises only `json_tuple_payload_v1` with a valid
identity envelope.

- [x] Strict mode: assert production path returns
  `SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED`. Evidence:
  `test_ec_spire_customscan_tuple_transport_retired_live_sql`.
- [x] Degraded mode: assert the upgrade hint is readable through
  `ec_spire_remote_search_degraded_skip_report` with the expected
  capability name (`pg_binary_attr_v1`). Evidence:
  `test_ec_spire_customscan_tuple_transport_retired_live_sql`.

### 12c.2.c: `local_statement_timeout` end-to-end

Today only `transport_faults.rs:217` covers the probe; the
user-facing query path is untested.

- [x] `SET statement_timeout` to a value below expected scan
  duration; run CustomScan query; assert cancel error. Evidence:
  `test_ec_spire_customscan_local_statement_timeout_sql`.
- [x] Assert no leaked transport state (libpq connection returned
  to pool, no orphaned prepared rows on remote). Evidence:
  `test_ec_spire_customscan_local_statement_timeout_sql` clears the
  timeout, routes the descriptor back to the normal loopback
  endpoint, and asserts the next CustomScan succeeds.

### 12c.2.d: `stale_remote_epoch_manifest` end-to-end

Today only contract pinning at `epoch_manifest.rs:78`.

- [x] Remote advertises a manifest version behind `active_epoch`.
  Evidence: `test_ec_spire_remote_epoch_manifest_persist_ready`.
- [x] Strict-mode read: assert
  `endpoint_status = stale_remote_epoch_manifest`. Evidence:
  `test_ec_spire_remote_epoch_manifest_persist_ready`.
- [x] Assert the matrix-row action fires (refresh request or
  fail-closed per matrix). Evidence:
  `test_ec_spire_remote_epoch_manifest_persist_ready`.

### 12c.2.e: `remote_oom`

Covered by `src/tests/remote_search/transport_faults.rs`.

- [x] Decide: live fixture or accepted-deferral row.
- [x] If live: simulate remote OOM (e.g., issue a query that
  exceeds remote `work_mem` deliberately); assert matrix-row
  action fires.
- [x] If deferred: record the deferral rationale with reviewer
  acceptance. Not applicable: the row was not deferred; live fixture
  evidence in packet `707` was accepted by reviewer feedback `31090`.

### 12c.2.f: `simulated_network_partition`

Covered by `src/tests/remote_search/transport_faults.rs`.

- [x] Decide: live fixture or accepted-deferral row.
- [x] If live: drive an unreachable transport endpoint; assert
  detection and matrix-row action.
- [x] If deferred: record the deferral rationale with reviewer
  acceptance. Not applicable: the row was not deferred; live fixture
  evidence in packet `709` was accepted by reviewer feedback `31100`.

## Phase 12c.3: Stage E Lifecycle Matrix — Live Coverage (P1)

Source: audit Axis C. Six lifecycle rows at
`production_summary.rs:289-298` are contract-only.

### 12c.3.a: `drop_remote_index_in_flight`

- [x] Strict mode: long-running CustomScan + `DROP INDEX` on remote
  mid-scan; assert matrix-prescribed strict action fires.
- [x] Degraded mode: same; assert
  `degraded_skipped_dispatch_count` and the skip-category.

### 12c.3.b: `drop_remote_index_pre_dispatch`

- [x] `DROP INDEX` on remote before dispatch; assert
  pre-dispatch validation refuses with the expected category.
- [ ] Assert no remote SQL is issued (no descriptor refresh
  attempted against the dropped object).

### 12c.3.c: `reindex_remote_index_in_flight`

- [x] Strict mode: long-running CustomScan + `REINDEX INDEX` on
  remote mid-scan; assert matrix action.
- [x] Degraded mode: same; assert degraded skip reporting.

### 12c.3.d: `reindex_remote_index_pre_dispatch`

- [x] `REINDEX` before dispatch; assert pre-dispatch validation
  fires (or accepts, per matrix).
- [x] Assert descriptor freshness check picks up the new relfilenode
  if the matrix requires that.

### 12c.3.e: `create_index_concurrently_new_descriptor`

- [x] `CREATE INDEX CONCURRENTLY` on remote; assert descriptor
  refresh contract picks up the new index without errors.
- [x] Assert subsequent CustomScan uses the refreshed descriptor.

### 12c.3.f: `create_index_concurrently_pre_dispatch`

- [x] Same as above, with the CIC operation completed before any
  dispatch attempt; assert the new descriptor is the one used.

## Phase 12c.4: Schema Drift on the READ Path (P1)

Source: audit Axis C / 12a.6. The 12a.6 fingerprint guard
(`remote_candidates.rs:159`-ish, `validate_remote_write_shape_fingerprint`)
landed for INSERT/UPDATE/DELETE. The CustomScan read path has no
drift fixture.

### 12c.4.a: Coord-only drift on READ

- [ ] `ALTER TABLE` coordinator side only.
- [ ] Attempt CustomScan; assert pre-dispatch validation fires
  with `SPIRE_REMOTE_STATUS_SCHEMA_DRIFT`.
- [ ] Assert hint string names the coordinator as the drifted side.

### 12c.4.b: Remote-only drift on READ

- [ ] `ALTER TABLE` remote side only.
- [ ] Attempt CustomScan; assert pre-dispatch validation fires.
- [ ] Assert hint string names the remote as the drifted side.

### 12c.4.c: Both-sides drift on READ

- [ ] Independent `ALTER TABLE` on both coordinator and remote.
- [ ] Assert pre-dispatch validation fires.
- [ ] Assert hint string names both sides.

## Phase 12c.5: 2PC In-Doubt Reaper Coverage (P1)

Source: audit Axis C. The reaper happy path is covered
(`catalog_cleanup_policy.rs:591`); the `prepare_acked → commit_local`
in-doubt path is not.

### 12c.5.a: Coordinator-crash-mid-2PC reaper fixture

Simulate a coordinator backend exit after `prepare_acked` but before
`commit_local`.

- [x] Set up intent row in `prepare_acked` state with a dead
  coordinator xid; remote has matching prepared txn.
- [x] Invoke `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)`.
- [x] Assert orphan in `prepare_acked` with dead coord xid is
  rolled back (`ROLLBACK PREPARED` issued).
- [x] Set up parallel intent row in `commit_local` state with a
  dead coordinator xid.
- [x] Assert reaper preserves `commit_local` rows (operator
  escalation, not silent rollback).

### 12c.5.b: Intent state-machine invariants

Assert the intent state machine cannot bypass states.

- [x] Cannot transition `prepare_requested → commit_local`
  directly (must cross `prepare_acked`).
- [x] Cannot transition `prepare_requested → rollback_local`
  silently (must be either explicit rollback path or reaper).
- [x] Add an `#[cfg(test)]` invariant assertion in the intent
  update path so any future state-machine change fails the test.

## Phase 12c.6: Recall / Correctness Pinning (P1)

Source: audit Axis I. The only recall evidence is the single-query
spot check from packet `30980`; no SPIRE-side test pins recall in CI.

### 12c.6.a: SPIRE `recall@k=1.0` baseline fixture

Evidence: packet `705`, accepted in reviewer feedback `31090`.
`test_ec_spire_recall_at_10_matches_exact_on_full_probe` builds a
deterministic 64-row corpus, runs the indexed top-10 query with
`nprobe = nlists`, compares it with the exact top-10 reference, and
asserts returned ids are unique.

- [x] Build a small corpus (e.g., N=64) with embeddings + brute-force
  reference.
- [x] Run CustomScan with K=10; capture predicted set.
- [x] Assert predicted set equals brute-force set
  (recall@10 = 1.0).
- [x] Assert returned PIDs are unique (no duplicates from fanout).

### 12c.6.b: `nprobe` sweep recall fixture

Evidence: packet `705`, accepted in reviewer feedback `31090`.
`test_ec_spire_nprobe_sweep_recall_is_monotonic` uses the same
deterministic 64-row corpus shape and sweeps session `ec_spire.nprobe`
over `1, 4, 8, 16`.

- [x] Run the same corpus with `nprobe = 1`.
- [x] Run with `nprobe = 4`.
- [x] Run with `nprobe = 8`.
- [x] Run with `nprobe = 16`.
- [x] Assert recall is monotonically non-decreasing across the
  sweep (or pin acceptable plateau with reviewer-accepted threshold).

### 12c.6.c: Sign convention pin extension

Today `coordinator/tests.rs:115`
(`remote_heap_exact_score_uses_orderby_negative_inner_product`) pins
one 2-dim case.

- [x] Add a high-dim (≥128) case with known expected score.
- [x] Add a NaN-input rejection assertion (AM must refuse).
- [x] Add a dimension-mismatch error assertion (query dim ≠ index
  dim must produce a clear error).

## Phase 12c.7: Multi-Remote Fanout Coverage (P1)

Source: audit Axis B. `transport_faults.rs:2` is the only fanout
test; it asserts overlap, not parallelism. All loopback CustomScan
fixtures are 1-remote.

### 12c.7.a: Fanout=3 CustomScan fixture

Evidence: `test_ec_spire_customscan_three_remote_fanout_sql` maps
three selected coordinator PIDs to three loopback remote node IDs,
probes each remote endpoint for its expected origin-coded payload, and
asserts CustomScan returns the same union. The widening variant is
covered by `test_ec_spire_customscan_eight_remote_fanout_sql`.

- [x] Set up three loopback remotes with disjoint PID partitions.
- [x] Run CustomScan; capture returned rows with origin-remote
  metadata.
- [x] Assert all three remotes contributed at least one row.
- [x] Assert union of returned PIDs equals expected union.
- [x] Add a fanout=8 widening variant (P3, can defer) to detect
  scaling regressions.

### 12c.7.b: Selected-PID round-trip assertion

Extend `custom_scan.rs:46` (or add adjacent test).

- [x] Insert N=8 rows with known PID-to-payload mapping.
- [x] Run CustomScan with LIMIT 8.
- [x] Assert returned remote rows include exactly the selected PIDs
  (set equality, not just "≥0 rows returned").
- [x] Assert each returned row's payload matches the PID-to-payload
  mapping (catch payload-PID swap regressions).

## Phase 12c.8: Concurrency / Long-Running Scan Coverage (P1)

Source: audit Axis E.

### 12c.8.a: Concurrent DELETE collision against same PK

- [x] Fire two parallel coordinator-routed DELETEs against the same
  PK; assert exactly one succeeds.
- [x] Loser assertion: assert v1 contract (accepted with
  `deleted_count=0`, or whichever shape ADR-069 documents).
- [x] Assert no orphan placement rows or prepared-xact intent
  rows remain.

### 12c.8.b: Long-scan + DROP INDEX (coordinator side)

- [x] Start a long-running CustomScan in one session.
- [x] Issue `DROP INDEX` in another session against the coordinator
  index.
- [x] Assert the matrix-relevant coordinator-side category:
  PostgreSQL relation locks make coordinator `DROP INDEX` wait behind
  the active CustomScan and fail the DDL with `lock_timeout`, while
  the in-flight scan completes instead of being asynchronously
  unwound.
  Evidence: `test_ec_spire_customscan_coord_drop_waits_for_scan_sql`.
- [x] Assert no leaked transport state (libpq connection returned,
  no orphaned remote prepared rows).

### 12c.8.c: Long-scan + remote restart

- [x] Start a long-running CustomScan.
- [x] Restart the remote PG instance mid-scan.
  Evidence: `test_ec_spire_customscan_remote_backend_termination_rejoin_sql`
  simulates the restart as remote backend termination during candidate
  receive.
- [x] Strict mode: assert detection and matrix-prescribed action.
- [x] Degraded mode: assert degraded skip reporting.
- [x] Assert subsequent CustomScan can succeed after remote
  rejoins (no stale connection cached).

### 12c.8.d: Idle-in-transaction timeout during open CustomScan cursor

- [x] Open a cursor over CustomScan; do not read.
- [x] `SET idle_in_transaction_session_timeout` to a short value.
- [x] Assert backend disconnects per timeout.
- [x] Assert cursor close + cleanup runs (no leaked state).

## Phase 12c.9: DML Frontdoor Coverage Tightening (P2)

Source: audit Axis D.

### 12c.9.a: Non-PK SELECT pass-through end-to-end

Today only the hook-installation row exists (`dml_frontdoor.rs:2`).
Packet `30980` follow-up.

- [x] Drive a non-PK predicate SELECT against a SPIRE-fronted table.
- [x] Assert the chosen plan is Index Scan or Seq Scan (not a
  CustomScan).
- [x] Assert returned rows match the expected non-PK predicate.

### 12c.9.b: Composite-PK rejection

- [x] Define a table with a composite PK; attempt SPIRE registration.
- [x] Assert rejection with the expected category.

### 12c.9.c: Float PK rejection

- [x] Define a table with `float4`/`float8` PK; attempt SPIRE
  registration or DML.
- [x] Assert rejection.

### 12c.9.d: Numeric-out-of-int8 PK rejection

- [x] Coerce a `numeric` value outside the `int8` range into the
  PK predicate.
- [x] Assert rejection at classifier time, not at SPI execution.

### 12c.9.e: Tighten DELETE-idempotent contract shape

Today `dml_frontdoor.rs:2323` asserts only row counts.

- [x] Pin the response shape: `accepted=true`, `deleted_count=0`
  on idempotent re-DELETE.
- [x] Assert no remote DML is issued on the second DELETE.

### 12c.9.f: Split UPDATE/DELETE schema-drift into 3 variants

Evidence: `test_ec_spire_update_schema_drift_variants_sql` and
`test_ec_spire_delete_schema_drift_variants_sql` cover the split
matrix in `dml_schema_drift.rs`, keeping `dml_frontdoor.rs` under the
test-file size target.

- [x] UPDATE coord-only drift: assert hint names coordinator.
- [x] UPDATE remote-only drift: assert hint names remote.
- [x] UPDATE both-sides drift: assert hint names both.
- [x] DELETE coord-only drift.
- [x] DELETE remote-only drift.
- [x] DELETE both-sides drift.

### 12c.9.g: Tighten descriptor-race test

Evidence: `test_ec_spire_insert_descriptor_race_sql` now lives in
`insert_remote_trigger.rs` to keep `insert.rs` under the test-file
size target. It records the winning descriptor generation, pins the
winner/loser placement summary, and asserts no prepared-xact intent
rows remain in a non-terminal state.

- [x] Assert which descriptor generation won (record generation
  numbers).
- [x] Assert zero orphan placement rows after the race resolves.
- [x] Assert zero SPIRE prepared-xact intent rows in non-terminal
  state.

## Phase 12c.10: EXPLAIN / Cost / Planner Tightening (P2)

Source: audit Axes A, G.

### 12c.10.a: Tighten JSON-EXPLAIN assertions

Today `custom_scan.rs:188-208` uses substring asserts.

- [x] Add `EXPLAIN (ANALYZE, FORMAT JSON)` run.
- [x] Assert `"Actual Rows"` field present and equal to LIMIT.
- [x] Assert `"Actual Loops"` field present and equal to 1.
- [x] Assert `"Actual Total Time"` field present and > 0.

### 12c.10.b: Tighten cost-monotonicity tests to ratios

Today `custom_scan/tests.rs:351-427` uses loose `>` comparisons; a
flipped-sign fanout regression would slip past `high > low` if the
constant term dominates.

- [x] Fanout proportionality: assert `cost(fanout=N) /
  cost(fanout=1)` is within an expected band proportional to N
  (not just `cost(fanout=N) > cost(fanout=1)`).
- [x] Row-count proportionality: same shape across rows.
- [x] Payload-width proportionality: same shape across widths.

### 12c.10.c: Cost-GUC override EXPLAIN reflection

- [x] `SET ec_spire.cost_routing_dimension_scale` to 2x default;
  run EXPLAIN; assert cost increased proportionally.
- [x] Same for `cost_leaf_dimension_scale`.
- [x] Same for `cost_index_page_scale`.
- [x] Same for `cost_local_store_page_fanout_scale`.
- [x] Same for `cost_storage_scoring_multiplier`.
- [x] Same for `cost_rerank_multiplier`.

### 12c.10.d: Empty-placement planner-refusal positive fixture

Today `custom_scan.rs:455` returns `eligible=false` but no test
asserts what plan the planner produces in this case.

- [x] Create a SPIRE-fronted table with no active epoch.
- [x] Run a query; capture EXPLAIN.
- [x] Assert plan node is Index Scan or Seq Scan, not CustomScan.

### 12c.10.e: EXPLAIN ANALYZE counter contract pin

- [x] Snapshot the full `EXPLAIN (ANALYZE, FORMAT JSON)` output for
  a canonical query.
- [x] Pin the set of fields (not values) the CustomScan emits.
- [x] Document the field-set contract in a code comment so future
  changes are explicit.

## Phase 12c.11: Isolation Coverage Completion (P2)

Source: audit Axis F.

### 12c.11.a: Add `READ COMMITTED` isolation row

Today `catalog_cleanup_policy.rs:839-840` covers only
`REPEATABLE READ` and `SERIALIZABLE`.

- [x] Extend the matrix iterator to include `READ COMMITTED`.
- [x] Pin the expected v1 behaviour for distributed PK SELECT under
  `READ COMMITTED`.

### 12c.11.b: EvalPlanQual / stale-row pin test

Pin the documented contract from `begin_exec.rs:336-338`.

- [ ] Session A: `SELECT FOR UPDATE` over CustomScan; pause.
- [ ] Session B: UPDATE the same row, commit.
- [ ] Session A: resume; assert the documented stale-read outcome
  (recheck returns true, stale row surfaced).
- [x] Cross-reference the contract comment from
  `begin_exec.rs:420-428` in the test. Evidence:
  `test_ec_spire_remote_pk_select_isolation_contract_sql`.

## Phase 12c.12: Typed Tuple Transport Coverage (P2)

Source: audit Axis B.

### 12c.12.a: Empty projection list typed-payload

- [x] Pin the zero-attr typed payload bytes layout (today only the
  JSON column-list path is asserted).
- [x] Assert the empty metadata + value arrays are aligned per the
  protocol spec.

### 12c.12.b: Composite-only typed-payload

Today `tuple_heap.rs:280` mixes domain + composite.

- [x] Add a fixture with a pure composite (no domain wrapper).
- [x] Assert round-trip integrity through the typed path.

### 12c.12.c: Tighten null-array wire-byte assertion

Today `tuple_heap.rs:202` asserts round-trip success only.

- [x] Capture the bytes-on-wire for a NULL `text[]`.
- [x] Assert the negative-length sentinel encoding (not zero-length).
- [x] Add a regression-defense byte-pattern assertion so an encoder
  that wrote zero-length instead of NULL would fail.

## Phase 12c.13: Operator-Surface / Diagnostic Snapshot Coverage (P2)

Source: audit Axis K.

### 12c.13.a: Stage E matrix executor assertions

Today `production_summary.rs:234,290` is contract-only.

- [x] For each fault-matrix row prescribing a `fail_closed` action,
  drive a fixture that triggers the fault and assert
  `fail_closed` actually fires. Evidence:
  `test_ec_spire_prod_transport_network_partition_matrix_actions`,
  `test_ec_spire_prod_transport_remote_stmt_timeout_matrix_actions`,
  `test_ec_spire_prod_transport_remote_oom_matrix_actions`,
  `test_ec_spire_prod_transport_backend_terminated_matrix_actions`,
  `test_ec_spire_prod_transport_query_cancel_matrix_actions`,
  `test_ec_spire_prod_receive_drop_index_in_flight`,
  `test_ec_spire_prod_receive_reindex_in_flight`,
  `test_ec_spire_customscan_local_statement_timeout_sql`,
  `test_ec_spire_customscan_remote_backend_termination_rejoin_sql`,
  `test_ec_spire_customscan_tuple_transport_retired_live_sql`,
  `test_ec_spire_customscan_large_text_projection_cap_sql`, and
  `production_executor_heap_tuple_transport_retired_matrix_actions`.
- [x] For each row prescribing `skip_and_report`, drive a fixture
  and assert the degraded skip path is taken. Evidence:
  `test_ec_spire_prod_transport_network_partition_matrix_actions`,
  `test_ec_spire_prod_transport_remote_stmt_timeout_matrix_actions`,
  `test_ec_spire_prod_transport_remote_oom_matrix_actions`,
  `test_ec_spire_prod_transport_backend_terminated_matrix_actions`,
  `test_ec_spire_prod_transport_query_cancel_matrix_actions`,
  `test_ec_spire_prod_receive_drop_index_in_flight`,
  `test_ec_spire_prod_receive_reindex_in_flight`,
  `test_ec_spire_customscan_remote_backend_termination_rejoin_sql`,
  `test_ec_spire_customscan_tuple_transport_retired_live_sql`, and
  `production_executor_heap_tuple_transport_retired_matrix_actions`.
- [x] Cross-reference the executor test from the contract-pin
  comment so a reader sees both. Evidence:
  `test_ec_spire_stage_e_fault_matrix_contract` now names the
  executor-action fixture files.

### 12c.13.b: Diagnostic snapshot survival under DROP INDEX

Assert each diagnostic snapshot returns empty cleanly when invoked
against a dropped index (not panic / not stale data).

- [x] `ec_spire_index_hierarchy_snapshot`.
- [x] `ec_spire_index_object_snapshot`.
- [x] `ec_spire_index_delta_snapshot`.
- [x] `ec_spire_index_health_snapshot`.
- [x] `ec_spire_index_leaf_snapshot`.
- [x] `ec_spire_index_placement_snapshot`.
- [x] `ec_spire_index_scan_pipeline_snapshot`.
- [x] `ec_spire_index_top_graph_snapshot`.
- [x] `ec_spire_index_allocator_snapshot`.
- [x] `ec_spire_index_boundary_replica_placement_snapshot`.

### 12c.13.c: `ec_spire_relation_storage_snapshot` under REINDEX

- [x] Start REINDEX in one session. Evidence:
  `test_ec_spire_relation_storage_snapshot_during_reindex_sql`.
- [x] Call snapshot mid-REINDEX from another session. Evidence:
  `test_ec_spire_relation_storage_snapshot_during_reindex_sql`.
- [x] Assert sane behaviour (no panic, returns either pre-REINDEX
  state or `not_available` per documented contract). Evidence:
  `test_ec_spire_relation_storage_snapshot_during_reindex_sql`.

## Phase 12c.14: Data-Shape Edge Cases (P3)

Source: audit Axis J.

### 12c.14.a: Single-row corpus scan fixture

- [x] Build with N=1; run CustomScan; assert the one row is
  returned cleanly.

### 12c.14.b: All-duplicate-vector corpus

- [x] Build a corpus where all vectors are identical.
- [x] Run CustomScan with K=10; assert all top-K rows have
  identical scores.
- [x] Assert `recall@k=1.0` against brute-force.

### 12c.14.c: Numerical-extreme vector handling

- [x] Subnormal vector components: assert clean processing
  (no panic, no NaN propagation).
- [x] Magnitudes near `f32::MAX`: assert no overflow.
- [x] NaN component rejection: AM must refuse insertion.
- [x] `+Inf` / `-Inf` component rejection: AM must refuse insertion.

### 12c.14.d: Text-with-NUL-byte projection round-trip

- [x] Insert a row with `text` containing an embedded NUL byte.
- [x] Read through CustomScan; assert the NUL byte is preserved
  (or documented unsupported with explicit error).

### 12c.14.e: Very-large-string projection (≥1 MB)

- [x] Insert a row with a 1 MB text projection column.
- [x] Read through CustomScan; assert success up to
  `ec_spire.max_remote_payload_bytes_per_row`.
- [x] Insert a row exceeding the cap; assert
  `SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE` (boundary
  cross-check with 12c.2.a).

### 12c.14.f: Wide projection (≥32 columns)

- [x] Build a table with ≥32 projection columns.
- [x] Run CustomScan; assert recall@k matches brute-force.
- [x] Assert typed transport handles the width without truncation.

## Phase 12c.15: Multi-Store / Multi-NVMe Width (P3)

Source: audit Axis H.

### 12c.15.a: Three-store scan fixture

- [x] Build with 3 local stores.
- [x] Run scan; assert all 3 stores are touched.
- [x] Assert per-store counter rows match overall counters
  (route, candidate, byte sums).

### 12c.15.b: Four-store scan fixture

- [x] Build with 4 local stores.
- [x] Run scan; assert all 4 stores are touched.

### 12c.15.c: Sequential-backend label standalone test

Today embedded in larger fixtures.

- [x] Run scan; capture `local_store_execution_snapshot`.
- [x] Assert `local_store_execution_mode = 'sequential_backend'`.
- [x] Assert `local_store_parallelism_next_step =
  'async_or_parallel_store_group_executor'`.

## Phase 12c.16: Semantic Tightening Sweep (P2)

Source: audit "Semantic concerns" section.

### 12c.16.a: Tighten marquee CustomScan test

`custom_scan.rs:46` inserts 2 rows; this work expands it.

- [x] Insert N=8 rows with known PID-to-payload mapping.
- [x] Run CustomScan with LIMIT 8.
- [x] Assert returned-PID set equals selected-PID set.
- [x] Assert each row's payload matches the PID-to-payload mapping
  (catch payload-PID swap regressions; cross-link 12c.7.b).

### 12c.16.b: Tighten empty-remote-result test

`custom_scan.rs:288` pins JSON status but not invocation counts.

- [x] Add assertion that `EndCustomScan` was invoked exactly once.
- [x] Add assertion that pfree counters returned to baseline.

### 12c.16.c: Document Stage E contract-only status

`production_summary.rs:234,290`.

- [x] Add docstring on the fault-matrix contract test stating that
  it is contract-only and pointing at 12c.2 / 12c.13 for the live
  executor assertions.
- [x] Same for the lifecycle-matrix contract test (pointing at
  12c.3).

## Suggested Packet Sequence

P1 items first, ordered to land the load-bearing pins before any
Phase 13 work touches the surface. The DML-frontdoor and
EXPLAIN/cost tightenings (12c.9, 12c.10) can interleave with P1
where the same file is being edited.

1. `12c.1` CustomScan FFI lifecycle — closes the rescan/end/recheck
   gaps the prior audits kept flagging.
2. `12c.5` reaper in-doubt path — load-bearing for cross-AZ Phase 13.
3. `12c.2` Stage E fault-matrix live coverage — closes the new
   12a-era categories.
4. `12c.4` schema drift on READ.
5. `12c.7` multi-remote fanout and selected-PID round-trip.
6. `12c.6` recall pinning.
7. `12c.8` concurrency / long-scan coverage.
8. `12c.3` Stage E lifecycle live coverage — largest fixture
   investment, can run in parallel with the others.
9. `12c.9` DML frontdoor tightening.
10. `12c.10` EXPLAIN / cost / planner tightening.
11. `12c.11` isolation completion.
12. `12c.12` typed-transport coverage.
13. `12c.13` operator-surface / diagnostic survival.
14. `12c.16` semantic tightening sweep.
15. `12c.14` data-shape edge cases.
16. `12c.15` multi-store width.

## Exit Criteria

- Every CustomScan FFI callback (`CreateCustomScanState`,
  `BeginCustomScan`, `ExecCustomScan`, `EndCustomScan`,
  `ReScanCustomScan`, `ExplainCustomScan`, `recheck`) has at least
  one Rust-level or `#[pg_test]` fixture asserting observable state
  or output beyond "did not panic."
- Every row in the Stage E fault matrix and Stage E lifecycle matrix
  has either a live fixture or a reviewer-accepted deferral row
  with rationale.
- Schema-drift coverage exists for both write and read paths,
  with coord-only / remote-only / both-sides variants.
- 2PC reaper coverage includes the in-doubt `prepare_acked →
  commit_local` window, not just the lost-ack window.
- SPIRE recall has at least one CI-runnable assertion at
  `recall@k=1.0` on a small corpus and a sweep across nprobe.
- Multi-remote fanout >1 has at least one CustomScan fixture
  asserting all remotes contributed rows.
- The loose-assertion tests called out in the audit semantic-concerns
  list are either tightened or have an explicit reviewer-accepted
  rationale row for retaining the loose shape.
- No new production code in `src/am/ec_spire/` outside `#[cfg(test)]`
  blocks and minimum testability hooks (reviewer-confirmed).
- Phase 13 AWS verification may proceed under the same `30949`
  evidence-tier rules.
