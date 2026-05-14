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

- [ ] Cursor-rescan fixture: open cursor over CustomScan, fetch N/2
  rows, `MOVE FIRST`, fetch all, assert identical row set. Drives
  `ec_spire_rescan_custom_scan` (`begin_exec.rs:183`) end-to-end —
  the helper unit test at `custom_scan/tests.rs:316` does not.
- [ ] `EndCustomScan` palloc/pfree pairing fixture: cancel mid-Exec,
  verify `MemoryContextStats` returns to baseline. Exercises
  `begin_exec.rs:170` post-cancel cleanup path.
- [ ] `recheck` callback pin test: assert
  `ec_spire_custom_scan_recheck` always returns `true` and document
  the consequent stale-row contract from `begin_exec.rs:332-338`.
  A regression to `false` would silently drop rows during EvalPlanQual
  rerun.
- [ ] `MarkPos`/`RestrPos` planner-gate test: assert the planner
  rejects plans requiring Mark/Restore (e.g., MergeAppend over
  CustomScan or inner-rescan nested loop above it). Today the
  callbacks are `None` (`mod.rs:106-107`) but no test pins the
  planner-side exclusion.
- [ ] `BeginCustomScan` UPDATE/DELETE branch panic-recovery: drive
  `dml_update_value_exprs_from_plan` (`begin_exec.rs:90-92`) with
  invalid column metadata, assert panic during `Begin` does not leak
  half-initialized `SpireCustomScanExecState`.
- [ ] Replace the two `#[should_panic(expected="EcSpireDistributedScan
  production executor blocked")]` scaffolds at `custom_scan.rs:777,838`
  with positive assertions once the executor is wired. If the
  scaffolds still apply, document why and tighten the panic-expected
  string to a versioned identifier.

## Phase 12c.2: Stage E Fault Matrix — Live Coverage (P1)

Source: audit Axis C. The matrix at
`src/tests/remote_search/production_summary.rs:234` enumerates 11+
fault categories; only ~half have live fixtures. The 12a-era
categories have no live coverage at all.

- [ ] `payload_too_large` (12a.2) live fault fixture: drive the
  encoder to emit a payload exceeding the `ec_spire.max_remote_payload_bytes_per_row`
  GUC, assert strict mode returns
  `SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE`, degraded mode
  reports `degraded_skipped_dispatch_count`.
- [ ] `tuple_transport_retired` (12a.5) live fault fixture: stub a
  remote that advertises only `json_tuple_payload_v1` with valid
  identity envelope; assert production path returns
  `SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED` with the
  upgrade hint readable through
  `ec_spire_remote_search_degraded_skip_report`.
- [ ] `local_statement_timeout` end-to-end CustomScan read fixture:
  `SET statement_timeout = X`, run a CustomScan query that exceeds
  it, assert cancel error and no leaked transport state. Today only
  `transport_faults.rs:217` covers the probe; the user-facing query
  path is untested.
- [ ] `stale_remote_epoch_manifest` end-to-end strict-mode read:
  remote advertises a manifest version behind active_epoch, strict
  mode reads against it, assert
  `endpoint_status = stale_remote_epoch_manifest` and the matrix-row
  action fires. Today only contract pinning at `epoch_manifest.rs:78`.
- [ ] `remote_oom` live fixture (or accepted-deferral row with
  rationale). Today matrix row only.
- [ ] `simulated_network_partition` live fixture (or accepted-deferral
  row). Today matrix row only.

## Phase 12c.3: Stage E Lifecycle Matrix — Live Coverage (P1)

Source: audit Axis C. Six lifecycle rows at
`production_summary.rs:289-298` are contract-only.

- [ ] `drop_remote_index_in_flight`: long-running CustomScan + `DROP
  INDEX` on remote mid-scan; assert the matrix-prescribed strict /
  degraded behaviour fires.
- [ ] `drop_remote_index_pre_dispatch`: `DROP INDEX` on remote before
  dispatch; assert planner / pre-dispatch validation refuses.
- [ ] `reindex_remote_index_in_flight`: similar shape, `REINDEX
  INDEX` on remote mid-scan.
- [ ] `reindex_remote_index_pre_dispatch`: `REINDEX` before dispatch.
- [ ] `create_index_concurrently_new_descriptor`: `CREATE INDEX
  CONCURRENTLY` on remote; descriptor refresh contract.
- [ ] `create_index_concurrently_pre_dispatch`: same, ordered before
  dispatch.

## Phase 12c.4: Schema Drift on the READ Path (P1)

Source: audit Axis C / 12a.6. The 12a.6 fingerprint guard
(`remote_candidates.rs:159`-ish, `validate_remote_write_shape_fingerprint`)
landed for INSERT/UPDATE/DELETE. The CustomScan read path has no
drift fixture.

- [ ] Coord-only drift on READ: `ALTER TABLE` coordinator side only,
  attempt CustomScan, assert pre-dispatch validation fires with
  `SPIRE_REMOTE_STATUS_SCHEMA_DRIFT` and hint naming the coordinator.
- [ ] Remote-only drift on READ: `ALTER TABLE` remote side only,
  attempt CustomScan, assert pre-dispatch validation fires and hint
  names the remote.
- [ ] Both-sides drift on READ: independent drifts; assert the hint
  names both sides.

## Phase 12c.5: 2PC In-Doubt Reaper Coverage (P1)

Source: audit Axis C. The reaper happy path is covered
(`catalog_cleanup_policy.rs:591`); the `prepare_acked → commit_local`
in-doubt path is not.

- [ ] Coordinator-crash-mid-2PC fixture: simulate a coordinator
  backend exit after `prepare_acked` but before `commit_local`;
  invoke the reaper; assert it preserves rows in `commit_local`
  state (operator escalation) and rolls back orphans whose
  coordinator xid is no longer live but state is not `commit_local`.
- [ ] Add explicit transition-time invariants: assert the intent
  state machine cannot go directly from `prepare_requested` to
  `commit_local` without crossing `prepare_acked`.

## Phase 12c.6: Recall / Correctness Pinning (P1)

Source: audit Axis I. The only recall evidence is the single-query
spot check from packet `30980`; no SPIRE-side test pins recall in CI.

- [ ] SPIRE `recall@k=1.0` baseline fixture: small corpus (e.g.,
  N=64), brute-force vs CustomScan, K=10, assert set match.
- [ ] `nprobe` sweep recall fixture: `nprobe ∈ {1, 4, 8, 16}`,
  assert monotonic recall improvement (or pin acceptable plateau).
- [ ] Tighten `coordinator/tests.rs:115`
  (`remote_heap_exact_score_uses_orderby_negative_inner_product`)
  to add high-dim, NaN-input rejection, and dimension-mismatch
  error semantics. Today it pins one 2-dim case.

## Phase 12c.7: Multi-Remote Fanout Coverage (P1)

Source: audit Axis B. `transport_faults.rs:2` is the only fanout
test; it asserts overlap, not parallelism. All loopback CustomScan
fixtures are 1-remote.

- [ ] Fanout=3 CustomScan fixture: three loopback remotes, assert
  all three contribute returned rows and the union matches the
  expected PID set.
- [ ] Selected-PID round-trip assertion: extend
  `custom_scan.rs:46` (or add adjacent) so the returned remote rows
  include exactly the selected PIDs — a regression where remotes
  return arbitrary PIDs would fail.

## Phase 12c.8: Concurrency / Long-Running Scan Coverage (P1)

Source: audit Axis E.

- [ ] Concurrent DELETE collision against same PK: assert v1
  contract (accepted with `deleted_count=0` on the loser, or whichever
  shape ADR-069 documents).
- [ ] Long-scan + DROP INDEX (coordinator side) cancellation:
  assert the running CustomScan unwinds cleanly with no leaked
  transport state.
- [ ] Long-scan + remote restart: assert detection and degraded /
  strict behaviour per matrix.
- [ ] Idle-in-transaction timeout during open CustomScan cursor:
  assert cursor close + cleanup.

## Phase 12c.9: DML Frontdoor Coverage Tightening (P2)

Source: audit Axis D.

- [ ] Non-PK SELECT pass-through end-to-end (packet `30980`
  follow-up): drive a non-PK predicate, assert the original
  Index Scan / Seq Scan plan is preserved (not a CustomScan).
  Today only the hook-installation row exists.
- [ ] Composite-PK rejection in DML frontdoor.
- [ ] Float / numeric-out-of-int8 PK rejection.
- [ ] Tighten DELETE-idempotent test (`dml_frontdoor.rs:2323`) to
  pin the `accepted/deleted_count=0` contract shape from packet
  `30980`, not just row counts.
- [ ] Split the UPDATE/DELETE schema-drift test
  (`dml_frontdoor.rs:1672`) into coord-only, remote-only, and
  both-sides variants.
- [ ] Tighten descriptor-race test (`insert.rs:993`) to assert
  specific descriptor-generation outcome and zero orphan placements.

## Phase 12c.10: EXPLAIN / Cost / Planner Tightening (P2)

Source: audit Axes A, G.

- [ ] Tighten JSON-EXPLAIN assertions at `custom_scan.rs:188-208` to
  include `"actual rows"` and loop counts via `EXPLAIN ANALYZE`.
- [ ] Tighten cost-monotonicity tests
  (`custom_scan/tests.rs:351-427`) to assert proportional shape
  (ratios), not just `>` direction. A flipped-sign fanout regression
  would slip past `high > low`.
- [ ] Cost-GUC override EXPLAIN reflection fixture: `SET
  ec_spire.cost_*`, run EXPLAIN, assert costs changed.
- [ ] Empty-placement planner-refusal positive fixture: assert the
  plan node chosen is Index Scan / Seq Scan, not CustomScan.
- [ ] EXPLAIN ANALYZE counter contract: per-row execution counters
  (loops, actual rows) appear and have the expected shape.

## Phase 12c.11: Isolation Coverage Completion (P2)

Source: audit Axis F.

- [ ] Add `READ COMMITTED` row to the isolation matrix at
  `catalog_cleanup_policy.rs:839-840` so all three levels are
  asserted (not just `REPEATABLE READ` and `SERIALIZABLE`).
- [ ] EvalPlanQual / stale-row pin test: row updated by another tx
  between SELECT FOR UPDATE and the recheck callback; assert the
  documented stale-read outcome from `begin_exec.rs:336-338`.

## Phase 12c.12: Typed Tuple Transport Coverage (P2)

Source: audit Axis B.

- [ ] Empty projection list typed-payload fixture: zero-attr typed
  payload bytes layout pinned (today only the JSON column-list path
  is asserted).
- [ ] Composite-only typed-payload fixture (isolate from domain):
  today `tuple_heap.rs:280` mixes domain + composite.
- [ ] Tighten null-array typed-payload test (`tuple_heap.rs:202`)
  to assert wire bytes (negative-length sentinel), not just
  round-trip success.

## Phase 12c.13: Operator-Surface / Diagnostic Snapshot Coverage (P2)

Source: audit Axis K.

- [ ] Drive Stage E matrix executor assertions where the row
  prescribes an action (e.g., `remote_oom` → `fail_closed`); run a
  fixture and assert the action fires, not just the row exists.
- [ ] Diagnostic snapshot survival under DROP INDEX mid-call: assert
  every `ec_spire_*_snapshot(index_oid)` returns empty cleanly (does
  not panic) when invoked against a dropped index.
- [ ] `ec_spire_relation_storage_snapshot` invariants under
  REINDEX mid-flight: assert sane behaviour at the boundary.

## Phase 12c.14: Data-Shape Edge Cases (P3)

Source: audit Axis J.

- [ ] Single-row corpus scan fixture.
- [ ] All-duplicate-vector corpus + `recall@k=1.0` assertion.
- [ ] Numerical-extreme vector fixture: subnormals, magnitudes near
  `f32::MAX`, NaN / +Inf rejection where the AM should refuse.
- [ ] Text-with-NUL-byte projection column round-trip.
- [ ] Very-large-string projection (≥1 MB; boundary against
  `ec_spire.max_remote_payload_bytes_per_row`).
- [ ] Wide projection (≥32 columns) recall + transport fixture.

## Phase 12c.15: Multi-Store / Multi-NVMe Width (P3)

Source: audit Axis H.

- [ ] Three-store scan fixture.
- [ ] Four-store scan fixture.
- [ ] Sequential-backend label assertion as a standalone test (today
  embedded in larger fixtures).

## Phase 12c.16: Semantic Tightening Sweep (P2)

Source: audit "Semantic concerns" section.

- [ ] Tighten the marquee test `custom_scan.rs:46` to insert N=8 rows,
  LIMIT 8, and assert returned-PID set equals selected-PID set.
- [ ] Empty-remote-result test at `custom_scan.rs:288`: add explicit
  assertions that `EndCustomScan` was invoked exactly once and that
  pfree counters returned to baseline.
- [ ] Stage E contract tests at `production_summary.rs:234,290`:
  document the contract-only-not-live status in the test docstring
  so a reader does not infer live coverage from the assertion shape.

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
