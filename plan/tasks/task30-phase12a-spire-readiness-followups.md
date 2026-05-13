# Task 30 Phase 12a: SPIRE Readiness Follow-ups

Status: planned
Owner: coder1 / SPIRE distributed production-hardening track
Priority: 1 before Phase 13 AWS verification burns real cloud spend

## Goal

Address the residual risks identified by the final Phase 12 review
(packet `30982`) before Phase 13 AWS/RDS verification opens code paths
under cross-AZ fanout, flapping links, and untrusted-remote-payload
scale. Phase 12a is not a new feature phase; it converts five
explicitly-deferred items into early-Phase-13 prerequisites so Phase 13
verification work cannot silently regress Phase 12 closure.

## Entry State

- Phase 12 is closed. Tracker `task30-phase12-spire-production-hardening.md`
  has zero unchecked rows. Final bundle packet `30981` accepted.
- Final-review packet `30982` recorded three P2 and three P3 issues that
  do not block Phase 12 closure but should land before Phase 13 cloud
  spend begins.
- Source SHA at task creation: branch `task-30-spire` HEAD `0b0f62f5`.

## Non-Goals

- AWS/RDS-class scale verification. That is Phase 13.
- New SPIRE features. Phase 12a is hardening of already-shipped surface.
- Re-running the Phase 12 fixture matrix. Phase 12 evidence stands; this
  phase adds new evidence for the new safety nets only.
- Multi-coordinator HA, cross-shard embedding UPDATE moves, DDL window
  guards, async store-group executor. Those remain later-ADR scope.

## Phase 12a.1: Orphaned Prepared-Xact Reaper (P2)

Source: review packet `30982`, finding "In-doubt prepared txn on
PREPARE-network-timeout."

- [ ] Document the in-doubt window: if `PREPARE TRANSACTION` ack is lost
  after remote WAL-flush, the prepared row is not in `prepared_rows` in
  `run_insert_prepare_requests_with_local_cancel_source`
  (`src/am/ec_spire/root/remote_candidates.rs:3463-3472`), so the outer
  rollback sweep does not visit it. Add this to ADR-069 as a named
  failure mode.
- [ ] Record durable prepared-transaction intent on the coordinator so a
  reaper can decide commit vs rollback without consulting the remote
  `pg_prepared_xacts` row alone. Minimum metadata:
  `(index_oid, node_id, served_epoch, xid, gid, intent_state)` where
  `intent_state` ∈ {`prepare_requested`, `prepare_acked`, `commit_local`,
  `rollback_local`}.
- [ ] Add `ec_spire_reap_orphaned_remote_prepared_xacts(node_id)` that:
  - [ ] scans the remote's `pg_prepared_xacts` for `ec_spire_insert_*`
    GIDs;
  - [ ] joins each GID against coordinator intent metadata;
  - [ ] rolls back any GID whose coordinator top-transaction OID is no
    longer live and whose intent state is not `commit_local`;
  - [ ] returns one row per resolved GID with action taken.
- [ ] Add an operator-runnable entrypoint that sweeps all registered
  remotes in one call.
- [ ] Fixture: simulate a lost-ack window by injecting a panic between
  remote WAL-flush of `PREPARE TRANSACTION` and the libpq ack. Verify
  the reaper resolves the orphan with no manual SQL.
- [ ] Decide whether to run the reaper as a SPIRE background worker on
  a configurable interval, or leave it operator-driven. Record decision
  in ADR-069 and `docs/SPIRE_LIBPQ_RUNBOOK.md`.

## Phase 12a.2: Remote-Payload Size Caps (P2)

Source: review packet `30982`, finding "Unbounded coordinator memory on
adversarial remote payload."

- [ ] Add GUC `ec_spire.max_remote_payload_bytes_per_row` (default sized
  from the `30975` measurement plus a 4x safety margin; record the
  chosen number in packet evidence).
- [ ] Add GUC `ec_spire.max_remote_payload_rows_per_batch` (default
  sized from the local capacity profile in
  `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md`).
- [ ] Enforce both caps in
  `decode_remote_search_typed_tuple_payload_pg_row`
  (`src/am/ec_spire/root/remote_candidates.rs:9468-9581`) **before**
  per-row allocation; on breach, return a strict-failure category
  `SPIRE_REMOTE_STATUS_REMOTE_PAYLOAD_TOO_LARGE` with hint naming the
  GUC.
- [ ] Apply the same caps to `selected_pids: Vec<u64>` plumbing at
  `remote_candidates.rs:3582` and any other unbounded
  remote-controlled `Vec<...>` allocation on the coordinator side.
- [ ] Fixture: a fault-injection remote that returns one row exceeding
  the row cap, and a batch exceeding the batch cap; assert strict mode
  reports the new failure category, degraded mode reports
  `degraded_skipped_dispatch_count`, and no memory growth beyond the
  cap is observable.
- [ ] Document the caps in `docs/SPIRE_LOCAL_CAPACITY_TARGETS.md` and
  `docs/SPIRE_LIBPQ_RUNBOOK.md` (operator action: raise the GUC if a
  legitimate workload trips it, with a packet-local benchmark).

## Phase 12a.3: Cost-Constant GUCs (P2)

Source: review packet `30982`, finding "Cost constants hardcoded, not
GUC." Also raised by the `30976` reviewer for AWS recalibration.

- [ ] Convert the four `const` scales in `src/am/ec_spire/cost.rs:11-14`
  to GUCs with the current packet `30976` values as defaults:
  - [ ] `ec_spire.cost_routing_dimension_scale`
  - [ ] `ec_spire.cost_leaf_dimension_scale`
  - [ ] `ec_spire.cost_index_page_scale`
  - [ ] `ec_spire.cost_local_store_page_fanout_scale`
- [ ] Convert the scoring multipliers at `cost.rs:377` and `cost.rs:385`
  to GUCs (`ec_spire.cost_storage_scoring_multiplier`,
  `ec_spire.cost_rerank_multiplier`) with current defaults.
- [ ] Surface the active values in `ec_spire_admin_snapshot()` /
  `ec_spire_explain_snapshot()` so EXPLAIN-driven tuning reports the
  live values.
- [ ] Verify the modeled-cost rows from packet `30976` still reproduce
  under the new defaults (no behavioral change).
- [ ] Fixture: set each GUC to a non-default value, run
  `ecaz bench spire-pipeline --include-cost-snapshot`, assert the
  snapshot reflects the override.
- [ ] Document the knobs and the packet `30976` calibration baseline in
  `docs/SPIRE_DIAGNOSTICS.md` so Phase 13 recalibration has a starting
  point and audit trail.

## Phase 12a.4: Stage E Fault Matrix CI Wiring (P3)

Source: review packet `30982`, finding "Stage E fault matrix bitrot
risk." Tracker line 26 of the Phase 12 task implies CI gating; reality
is point-in-time archived evidence from packet `30895`.

- [ ] Pick the lightweight subset of the 11 fault cases that is
  feasible in GitHub Actions `services:` with two PG18 clusters. At
  minimum: `remote_statement_timeout`, `local_cancel`, `epoch_mismatch`,
  and one pre-dispatch blocker (incompatible-version). The other 7
  remain operator-runnable via `ecaz dev spire-multicluster fault-pg18`.
- [ ] Add a workflow job that runs the chosen subset on every PR
  touching `src/am/ec_spire/**`, `sql/**`, or
  `scripts/run_spire_multicluster_*.sh`.
- [ ] Confirm the workflow reuses the existing `ecaz dev
  spire-multicluster` wrappers from packets `30971`/`30978`; no new
  shell scripting.
- [ ] Amend the Phase 12 task tracker line 26 to add the parenthetical:
  "(matrix archived in `30895`; CI re-runs subset $LIST; full matrix is
  operator-runnable, not CI-gated)."
- [ ] Document the CI vs operator-runnable split in
  `docs/SPIRE_LOCAL_READINESS.md` so the evidence boundary stays honest.

## Phase 12a.5: Typed Transport Retirement Error Specificity (P3)

Source: review packet `30982`, finding "JSON fail-closed error category
is generic."

- [ ] Add a new strict-failure category
  `SPIRE_REMOTE_STATUS_TUPLE_TRANSPORT_RETIRED` distinct from
  `SPIRE_REMOTE_STATUS_ENDPOINT_IDENTITY_MISMATCH`.
- [ ] In `remote_tuple_payload_production_sql`
  (`src/am/ec_spire/root/remote_candidates.rs:8637-8645`), return the
  new category whenever the remote endpoint advertises a valid identity
  but does not advertise `pg_binary_attr_v1`. Reserve the existing
  identity-mismatch category for genuine identity-mismatch shapes.
- [ ] Attach an actionable hint string naming the required capability
  (`pg_binary_attr_v1`) and the upgrade path.
- [ ] Fixture: a remote stubbed to advertise only `json_tuple_payload_v1`
  with a valid identity envelope; assert the production path returns
  the new category with the hint, and the operator can read the hint
  from `ec_spire_remote_search_degraded_skip_report(...)`.
- [ ] Update `docs/SPIRE_LIBPQ_RUNBOOK.md` with the new category and
  the operator response (upgrade remote `ecaz` extension version).

## Phase 12a.6: Remote-Side Schema Fingerprint (P3)

Source: review packet `30982`, finding "Schema-drift fingerprint is
coordinator-only."

- [ ] Decide scope: full echo-back round-trip on every dispatch, or
  echo-back only on descriptor register/refresh. Record the decision
  rationale in ADR-069 with a fixture that demonstrates the chosen
  detection latency.
- [ ] Implement remote-side fingerprint computation using the same
  `(attnum, name, typid, typmod, collation, notnull)` tuple as
  `coordinator_write_current_shape_fingerprint`
  (`remote_candidates.rs:2579`).
- [ ] Echo the remote fingerprint on descriptor register/refresh; store
  on the descriptor row alongside
  `coordinator_insert_shape_fingerprint`.
- [ ] Pre-dispatch validation compares coordinator and remote
  fingerprints; on mismatch, return `SPIRE_REMOTE_STATUS_SCHEMA_DRIFT`
  with a remediation hint naming which side drifted.
- [ ] Fixture: `ALTER TYPE` on the remote without re-registering;
  assert pre-dispatch validation fires before remote SQL execution.
- [ ] Document the v1.x remote-side fingerprint contract in ADR-069 and
  the DDL ordering runbook section of `docs/SPIRE_LIBPQ_RUNBOOK.md`.

## Phase 12a.7: Tracker Phrasing Nit (P3 cosmetic)

- [x] Amend `plan/tasks/task30-phase12-spire-production-hardening.md`
  line 26 to clarify Stage E evidence shape:
  "Stage E fault matrix (11 cases) and lifecycle matrix (6 cases) pass
  against the CustomScan build in packet `30895` (matrix archived in
  `30895`; live re-run cadence is reviewer-requested, not CI-gated; see
  Phase 12a.4 for the CI subset)."

## Suggested Packet Sequence

The P2 items are pre-requisites; the P3 items are quality improvements
that can interleave.

1. `12a.1` orphaned prepared-xact reaper (P2; biggest 2PC risk).
2. `12a.2` remote-payload size caps (P2; cheap, single-PR scope).
3. `12a.3` cost-constant GUCs (P2; enables Phase 13 tuning iteration).
4. `12a.4` Stage E CI subset (P3; prevents Phase 13 silently regressing
   Phase 12).
5. `12a.5` typed-transport retirement error specificity (P3; runbook
   quality).
6. `12a.6` remote-side schema fingerprint (P3; defense in depth).
7. `12a.7` tracker phrasing nit (P3 cosmetic; batch with any other
   tracker edit).

## Exit Criteria

- The three P2 items (`12a.1`, `12a.2`, `12a.3`) are implemented with
  packet-local fixtures and reviewer-accepted evidence.
- The Stage E CI subset (`12a.4`) is wired into the workflow on every
  PR touching SPIRE surface, and the tracker phrasing nit (`12a.7`) is
  applied.
- The P3 quality items (`12a.5`, `12a.6`) are implemented or explicitly
  deferred with reviewer-accepted rationale.
- Phase 13 AWS verification may proceed under the same `30949`
  evidence-tier rules, repeating any Phase 12a deferrals in its
  evidence shape.
