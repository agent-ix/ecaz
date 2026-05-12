---
topic: spire-customscan-architecture-review
agent: reviewer
role: reviewer
model: claude-opus-4-7
date: 2026-05-12
stage: phase-11-stage-d-closeout
status: open
---

# Review Request: SPIRE CustomScan + ADR-069 Architecture Closeout

Reviewer-initiated cross-cutting review packet, written after Stage D
landed end-to-end. Tracks two priority follow-up themes the coder
should plan against before Phase 11.9 (AWS scale entry) opens:

1. **Hardening — non-happy-path tests, fixtures, and runbook gaps.**
2. **Performance hotspots — especially retiring the JSON tuple-payload
   bridge in favor of a typed transport.**

This packet is also the durable record of architectural strengths,
tradeoffs, and deferred items. It cross-references the per-slice
feedback files where each finding originated.

## Context: what landed

CustomScan distributed read path complete. ADR-069 v1 transparent
distributed DML (INSERT, UPDATE, DELETE, PK SELECT) live with
multicluster fixtures. Materialization catalog (ADR-064/065/066)
fully retired. Stage E fault matrix (11/11) and lifecycle matrix
(6/6) pass against the CustomScan build with zero fixture rewrites.

201 commits across `task-30-spire`, 183 reviewer feedback files, 92
packets reviewed.

## Architectural strengths (durable record)

- **Single integration point per query class.** Vector ORDER BY LIMIT
  and DML PK-equality both go through `EcSpireDistributedScan`,
  discriminated by a plan-mode tag in `custom_private[0]`. No mirror
  table, no coordinator-side stub rows. Storage scales with remote
  shards.
- **Layered scaffolding.** classifier → query-extraction →
  relation-context → primitive-plan → invocation. Each layer
  unit-testable in isolation. Plan-replace slice landed cleanly
  because every dependency was already typed and tested.
- **Fail-closed by default at every layer.** Unsupported DML shapes
  against a SPIRE-marked table error at planner time with the
  ADR-069 message; never silently fall through to the empty
  coordinator heap. This is the load-bearing safety property.
- **Stage E reused with zero fixture rewrites.** The ADR-067 design
  claim that executor-state-focused fixtures survive an
  integration-point change actually held.

## Theme 1 — Hardening (non-happy-path)

The current fixture set covers happy paths well. The following
non-happy paths have either no coverage or only indirect coverage.
Each one is a real production failure mode that an operator could
hit at AWS scale.

### H1 — Concurrent INSERT descriptor-generation race

**Origin:** packet 30836 P2.

`ec_spire_remote_node_descriptor` is updated under a monotonic
generation guard (`WHERE descriptor_generation < $new_gen`). Two
concurrent coordinator INSERTs targeting the same `(index_oid,
node_id)` race; the loser fails the gen-advance check, errors, and
its registered xact-abort callback should `ROLLBACK PREPARED` on
the remote.

**Open:**
- No fixture exercises a parallel-INSERT burst against the same
  remote.
- Acknowledged as v1 retry behavior in ADR-069, but the
  caller-visible error wording has no SQLSTATE — applications
  must parse free-text `ec_spire coordinator insert descriptor
  refresh did not advance descriptor_generation`.

**Fix shape:**
- Add a multicluster fixture that fires N parallel coordinator
  INSERTs against the same descriptor and asserts:
  - Exactly one succeeds.
  - All others abort cleanly (no orphaned remote prepared
    transactions).
- Pin a stable SQLSTATE for the refresh-race error so apps can
  catch programmatically.
- Document the retry contract (deterministic SQLSTATE → safe to
  retry) in ADR-069.

### H2 — Concurrent DELETE collision

**Origin:** packet 30839 P2.

The DELETE primitive errors when `remote_deleted_count != 1`. If
two coordinator DELETEs race on the same row, the loser sees `0`
and errors. ADR-069 hasn't pinned whether this should retry-into-
success or fail loudly.

**Fix shape:**
- Pick a policy in ADR-069 ("DELETE-not-found is success" vs
  "DELETE-not-found is error"). Strong preference for matching PG's
  default DELETE semantics: `0 rows affected` is success, and the
  primitive should treat `count == 0` as success while still
  removing the placement-directory row (idempotent cleanup).
- Add a concurrent-DELETE fixture asserting whichever policy is
  chosen.

### H3 — Coordinator backend crash mid-2PC

**Origin:** packet 30830 P2.

The prepared-transaction GID embeds the coordinator backend's pid:
`ec_spire_insert_<oid>_<node>_<epoch>_<xid>_<pid>`. If the
coordinator crashes between local commit and remote `COMMIT
PREPARED`, the remote orphans a prepared transaction whose pid
field is now meaningless for recovery.

**Open:**
- No operator runbook for "I see `ec_spire_insert_*` GIDs in
  `pg_prepared_xacts` on a remote — what now?"
- The pid-in-GID actively hurts recovery (operators have no way to
  correlate the dead pid with anything live).

**Fix shape:**
- Drop the pid from the GID format — the
  `(index_oid, node_id, served_epoch, xid)` tuple is unique enough.
- Add a runbook section: how to identify orphaned SPIRE prepared
  transactions, when to commit vs roll back, and how to verify the
  outcome from the coordinator side (likely via the placement
  directory — if the placement row was committed locally, commit
  the prepared remote; if not, roll back).
- Consider a coordinator-side helper:
  `ec_spire_recover_orphaned_prepared_xacts(node_id)` that scans
  the remote's `pg_prepared_xacts` and resolves them against the
  placement directory.

### H4 — Statement-timeout / cancel mid-INSERT-prepare

**Origin:** packet 30830 P2.

The INSERT 2PC dispatch uses synchronous `client.batch_execute`
for `BEGIN; INSERT; PREPARE TRANSACTION`. Unlike the read path's
async tokio-postgres dispatch, this path doesn't bridge PG's
backend interrupt machinery (`InterruptPending` /
`QueryCancelPending`) to the libpq cancel.

**Open:**
- A timeout fired while the remote is preparing leaves the local
  backend blocked.
- A `pg_cancel_backend` against a stuck coordinator backend may
  not propagate to the remote.

**Fix shape:**
- Bring the synchronous batch_execute path to parity with Stage C's
  async cancel-bridge pattern.
- Add a fixture: start an INSERT, simulate a remote slow-prepare
  (statement_timeout on the remote, or a packet-level delay), then
  cancel the local backend and assert the remote prepared
  transaction is rolled back, not orphaned.

### H5 — Visibility skew on PK SELECT (EvalPlanQual gap)

**Origin:** packet 30810 P2 + 30873.

`ec_spire_custom_scan_recheck` unconditionally returns `true`. The
CustomScan returns a virtual tuple from the remote and reports it.
If the remote heap mutated between primitive call and the
coordinator's executor processing the tuple (concurrent UPDATE),
the coordinator returns the now-stale value. PG's normal
EvalPlanQual machinery would re-visit the tuple.

**Open:**
- No fixture exercises EvalPlanQual against a distributed table.
- SERIALIZABLE / REPEATABLE READ semantics are silently weakened
  on distributed reads.

**Fix shape:**
- Document the v1 limitation in ADR-068 / ADR-069: distributed
  table reads provide read-committed semantics only; predicate
  locks and EvalPlanQual rerun do not extend to remote rows.
- Add a fixture that runs `BEGIN ISOLATION LEVEL SERIALIZABLE;
  SELECT ... WHERE id = $1; UPDATE; COMMIT` against a distributed
  table and asserts whatever behavior the v1 contract pins.
- Future: if SERIALIZABLE matters, the recheck callback would need
  to re-run the primitive with the per-snapshot context, which
  doubles read RTT in the worst case.

### H6 — Schema drift between coordinator and remote

**Origin:** packet 30835 P2.

The descriptor refresh checks index identity bytes, not the user
table's column shape. An operator who runs `ALTER TABLE` on the
coordinator but forgets the remote will:
- Hit a fingerprint mismatch on read (existing Stage B coverage).
- For INSERT: the `to_jsonb(NEW)` payload is built from the
  coordinator's column list; `jsonb_populate_record` on the remote
  silently drops unknown keys and inserts NULL for missing remote
  columns. May silently succeed with wrong column values.

**Fix shape:**
- Fingerprint the user table's column shape on the coordinator at
  trigger-install time; record on the descriptor.
- On INSERT, compare current coordinator column shape vs descriptor
  fingerprint; if drift detected, raise a clear "schema drift —
  re-register descriptor or align remote schema" error before the
  remote dispatch.
- Add a focused fixture that ALTER TABLEs the coordinator,
  attempts INSERT, and asserts the schema-drift error fires.

### H7 — `to_jsonb(NEW)` round-trip for non-trivial column types

**Origin:** packet 30835 P2 (partially closed by 30844).

bytea round-trip is fixture-locked (30844). Other types not yet
covered:
- `numeric` / `decimal` — JSON serializes as number; precision
  loss possible on round-trip.
- `timestamptz` — JSON ISO-8601 string; timezone preservation?
- `json` / `jsonb` columns themselves — nested JSON in the
  payload.
- `text` with embedded NUL or non-UTF-8 bytes — JSON spec says
  no NUL in strings; PG `text` allows them.
- `domain over base type` — does `jsonb_populate_record` honor
  domain checks?

**Fix shape:**
- One regression fixture per type listed, asserting byte-for-byte
  equality after round-trip through trigger → JSON → remote
  endpoint → remote heap.
- For types that can't round-trip cleanly (likely `text` with
  NULs), document as v1 trigger limitations.

### H8 — Param-bigint overflow and edge values at runtime

**Origin:** packet 30868 P2.

The classifier accepts int2/int4/int8 for PK predicates. Runtime
coercion via `DatumGetInt16/32/64` is lossless. Edge cases not
covered:
- `WHERE id = $1::numeric` where `$1` is `9223372036854775808`
  (i64::MAX + 1) — should reject as type mismatch (not int8).
- `WHERE id = $1::int8` with `$1 = NULL` — already covered by NULL
  rejection; worth a fixture confirming the SQLSTATE is stable.
- `WHERE id IN (1, 2)` — classifier should reject as
  "not bigint equality"; worth a fixture.
- `WHERE id = $1 OR id = $2` — same.

**Fix shape:**
- Negative-coverage fixture for each unsupported predicate shape.
  Asserts the classifier rejects with the expected SQLSTATE.

### H9 — `max_prepared_transactions` exhaustion

**Origin:** packet 30830 P2.

PostgreSQL's `max_prepared_transactions` GUC defaults to 0. The
SPIRE INSERT path requires it set on every remote. pg_test
sets it to 10; production defaults aren't enforced. When the cap
is hit:
- `PREPARE TRANSACTION` errors with PG's standard "maximum
  prepared transactions reached."
- The error surfaces to the application as a generic SQLSTATE
  53400 (configuration_limit_exceeded).
- There's no SPIRE-side guidance pointing operators at the GUC.

**Fix shape:**
- Document `max_prepared_transactions` as a required remote GUC in
  the operator runbook.
- Add a preflight check on remote-descriptor registration that
  reads `current_setting('max_prepared_transactions')` and warns
  if `< expected_concurrent_writers`.
- Wrap the `PREPARE TRANSACTION` error with a SPIRE-named hint
  pointing at the GUC.

### H10 — DDL inside DML window

**Origin:** general (no specific packet).

What happens if `ALTER TABLE ... ADD COLUMN` runs on the
coordinator while a coordinator-routed INSERT is in flight? The
trigger captures `to_jsonb(NEW)` at trigger time using the
coordinator's current schema; the remote dispatch may arrive at a
remote that hasn't yet seen the ADD COLUMN.

**Fix shape:**
- Document the ordering: DDL on the coordinator happens, then DDL
  on each remote, then resume writes. Mixing DDL with active
  writes is undefined.
- Stretch goal: a DDL guard that blocks INSERT during a recorded
  DDL window. Probably overkill for v1.

### H11 — Multi-row INSERT through trigger

**Origin:** packet 30831 P2.

`ec_spire_remote_insert_tuple_payload` is single-row only. The
`BEFORE INSERT FOR EACH ROW` trigger fires per row, so multi-row
INSERTs work but pay one prepared-transaction round-trip per row.
No fixture exercises this.

**Fix shape:**
- Fixture: `INSERT INTO distributed_tbl (id, ...) VALUES (1,
  ...), (2, ...), (3, ...)` and assert all three rows land on
  their respective remotes; assert the coordinator transaction
  prepared three remote transactions and committed all on local
  commit.
- Stage F: design the bulk-row variant of the endpoint
  (`ec_spire_remote_insert_tuple_payloads(...)` with
  `jsonb_populate_recordset`) for high-throughput multi-row
  INSERT.

### H12 — `ec_spire_placement` write contention

**Origin:** general (placement table is the central mutable state).

Every coordinator-routed INSERT and DELETE writes to
`ec_spire_placement`. PG's heap + index locking handles this for
correctness, but at high write throughput the placement table
becomes a contention point — same row updates serialize, and the
PK btree page splits become hot.

**Open:**
- No load test for placement-table contention.
- The placement table is unpartitioned.

**Fix shape:**
- Stage F load fixture: 1000 parallel INSERTs to distinct PKs,
  assert no deadlocks and bounded latency growth.
- Future: partition `ec_spire_placement` by `index_oid` so multi-
  table workloads don't share a single hot table.

## Theme 2 — Performance hotspots

Sorted by likely production impact, with the JSON protocol
retirement called out per the user's priority.

### P1 — Replace JSON tuple-payload bridge with typed transport

**Origin:** packets 30807, 30814, 30816, 30880 (multiple).

This is the biggest performance lever still open. Every
remote-origin tuple goes through:
- Remote: `to_jsonb(heap_row)` + projection filter → text JSON.
- Wire: text JSON over libpq.
- Coordinator: `serde_json::from_str` → per-attribute
  `InputFunctionCall` (cached `FmgrInfo` per attribute, so the
  fmgr setup is one-shot, but the per-row decode + datum
  construction allocates).

Costs:
- Remote CPU: full row → JSON serialize for every returned tuple,
  even when the projection is narrow.
- Wire bytes: JSON is verbose vs binary.
- Coordinator CPU: `serde_json` parse per row, per-attribute text
  → datum conversion via `InputFunctionCall`.
- Memory: per-row Vec<u8> allocation in `dml_frontdoor_bigint_pk_value_bytes`
  and per-row JSON String allocation.
- Coverage gap: array and composite columns rejected at the
  scalar gate (30816) — the JSON bridge can't express them safely.
- `serde_json` runtime dependency.

**Fix shape (recommended path):**

The cleanest replacement is **PostgreSQL's binary protocol for
composite types**:

1. Define a server-side composite type per requested row shape:
   `(col1 type1, col2 type2, ...)`. This composite is built per
   `BeginCustomScan` from the requested column list, registered
   in the remote endpoint's call.
2. Remote endpoint returns a single `record` column whose value
   is the row in PG's binary tuple format. PG already has
   `record_send` / `record_recv` for this; we don't have to
   invent a binary protocol.
3. Coordinator decodes via `record_recv` into a TupleTableSlot
   directly. No `serde_json`, no `InputFunctionCall` per
   attribute.
4. Array and composite columns work — PG's binary record format
   handles them natively.

Alternative: if defining per-shape composites is too dynamic,
use libpq's binary parameter mode for individual columns. Per
PG attribute, the remote sends `typsend` output as bytea; the
coordinator runs `typreceive` directly into the slot. Less
elegant than composite-records but doesn't require type
registration.

**Migration shape:**
- Land the typed endpoint
  (`ec_spire_remote_select_tuple_record(index_oid, pk_column,
  pk_value, requested_columns)`) alongside the existing JSON
  endpoint.
- Switch the CustomScan executor to prefer the typed endpoint
  when both ends advertise support; fall back to JSON.
- Once all production deployments confirm the typed path, drop
  the JSON endpoint and remove the `serde_json` dep.
- The scalar gate (30816) can be removed because typed transport
  handles arrays/composites natively.

**Expected gain:** 3-10x throughput on tuple-heavy workloads
(estimate based on Citus benchmarks of similar JSON→binary
migrations); array column support unblocked.

### P2 — Replace `ec_spire_placement` seqscan in planner gate

**Origin:** packet 30873 P2 (still open).

`custom_scan_index_has_sql_placement(index_oid)` runs
`table_open` + `table_beginscan_catalog` over `ec_spire_placement`
looking for any row matching `index_oid`. This is a sequential
heap scan, O(N) on placement table size, on every planner
invocation against an `ec_spire`-indexed relation.

**Fix shape:**
- Use `index_beginscan` against the placement table's `(index_oid,
  pk_value)` PK. Existence check becomes O(log N).
- OR use a per-snapshot cache keyed on `(heap_oid,
  relcache_invalidation_xid)` — eligibility is stable within one
  snapshot.
- OR add an `ec_spire_placement_index_oid_idx` covering index
  specifically for the existence check.

The PK-index approach is cleanest; the cache is the cheapest
follow-up.

### P3 — Per-attribute `FmgrInfo` cached but per-row decode still allocates

**Origin:** packet 30816 P2 (partially closed).

`FmgrInfo` cached per scan in 30816. But per row, per attribute,
`custom_scan_json_value_to_datum` still:
- Calls `value.to_string()` (allocates).
- Constructs a `CString` (allocates).
- Calls `InputFunctionCall` (PG runs the type's input function,
  often allocates).

**Fix shape:** subsumed by P1 — the typed transport replaces
`InputFunctionCall` with a direct binary deserialization.

### P4 — Catalog-backed relation context per planner call

**Origin:** packet 30856 P3.

Every plan against a SPIRE-eligible relation pays for `table_open`
+ `RelationGetIndexList` + per-index `index_open` to derive the
relation context. Cheap per-call (relcache hits) but multiplies
under high QPS.

**Fix shape:**
- Per-snapshot cache keyed on
  `(heap_oid, relcache_invalidation_xid)` storing the resolved
  `SpireDmlFrontdoorRelationContext`.
- Invalidate on `RelcacheInvalidateCallback` for the heap or any
  of its ec_spire indexes.

### P5 — 2PC commit latency on every coordinator-routed write

**Origin:** ADR-069 design choice (acknowledged tradeoff).

Pure cost of correctness: every INSERT/DELETE pays
`BEGIN; INSERT; PREPARE TRANSACTION; COMMIT PREPARED` round-trips.
At low write throughput this is fine; at 1k+ writes/sec the
prepared-transaction WAL overhead dominates.

**Fix shape (limited; this is the cost of distributed atomicity):**
- For applications that don't need cross-shard atomicity, the
  bulk-load primitives (`classify_centroid` +
  `register_placement_batch`) skip 2PC entirely. Document the
  perf tradeoff in the operator runbook.
- Future ADR: explore presumed-commit / presumed-abort variants
  of 2PC for SPIRE's specific workload.

### P6 — Symbolic cost model needs benchmark calibration

**Origin:** packet 30827 P2 (acknowledged).

`CUSTOM_SCAN_REMOTE_DISPATCH_CPU_UNITS = 32`,
`CUSTOM_SCAN_ROUTING_SCORE_BOUND = 64`,
`CUSTOM_SCAN_MERGE_CPU_UNITS = 4` are placeholders, not derived
from measurement. Mis-pricing affects planner choices when other
plans are competitive.

**Fix shape:** Stage F packet that runs the suite-config benchmark
across realistic remote fanout / placement counts and derives the
constants from latency measurements.

### P7 — JSON column-name lists in `custom_private`

**Origin:** packet 30880 P3.

`custom_private` stores column-name lists as JSON strings. Per
`BeginCustomScan` parse cost (small but multiplies on prepared-
statement re-execute).

**Fix shape:** native PG `T_String` list nodes (or `T_Integer` +
N strings positionally). Subsumed by future `custom_private`
layout cleanup once mode discriminator also moves off
`Oid::from(u32)` packing.

### P8 — Per-row `Vec<u8>` allocation for PK bytes

**Origin:** packet 30864 P3.

`dml_frontdoor_bigint_pk_value_bytes(i64) -> Vec<u8>` allocates
8 bytes + Vec metadata per call. For DML this is once per
statement (not per row); for the read path it's once per
candidate.

**Fix shape:** `[u8; 8]` stack allocation, or pass a `&mut [u8;
8]` writer. Cosmetic for current call sites; worth doing if
profiling shows allocation pressure.

### P9 — Synchronous INSERT 2PC dispatch (no async pipeline)

**Origin:** packet 30830 P2.

Wide-fanout INSERT (one transaction inserting N rows distributed
across M nodes) serializes M prepare RTTs. The Stage C async read
path has tokio-postgres pipeline support; the INSERT path uses
synchronous `batch_execute`.

**Fix shape:** migrate INSERT 2PC dispatch to the same async
adapter as the read path. Pipeline N PREPAREs, await all, then
issue COMMIT PREPAREDs in pipeline. Reduces wide-fanout INSERT
latency from O(M) to O(1) RTTs.

## Tradeoffs made (durable record)

The pivot's structural choices, in order of operator-visible cost:

| Tradeoff | Why | Cost |
|---|---|---|
| No `ModifyTable` on transparent UPDATE/DELETE | Citus-precedent planner-hook plan-tree replacement is the only way to handle remote-owned rows that don't exist in the coordinator heap. | No `RETURNING`, no row triggers, no statement transition tables on transparent distributed DML. Documented as v1 contract in ADR-069. |
| JSON tuple payload over libpq | First-cut shape that lets PG own type input via `jsonb_populate_record` and avoids inventing a binary protocol. | Per-row encode + decode overhead; serde_json runtime dep; rejects array/composite columns. **Theme 2 P1 retires this.** |
| Single-bigint PK only for v1 | int8send → 8-byte bytea matches placement column shape; composite/UUID needs separate encoder + classifier work. | Operators with `id uuid` or composite PKs see "v1 requires single bigint PK" errors. |
| Coordinator-routed writes (no application sharding) | ADR-069 explicit operator preference. | Coordinator is a write throughput bottleneck and SPOF. Bulk-load escape hatch exists. |
| 2PC for every INSERT/DELETE | Without 2PC, placement directory and remote heap can drift on coordinator crash. | One extra remote RTT per write; `max_prepared_transactions` GUC dependency; manual recovery path. |
| No 2PC on UPDATE / PK SELECT | UPDATE doesn't change placement; PK SELECT is read-only. | At-most-once UPDATE under network partition; idempotent UPDATEs are safe; counter-style needs app idempotency. |
| No row-dependent SET expressions | Requires NEW/OLD row evaluation which the CustomScan path doesn't carry. | `SET counter = counter + 1` fails closed; operators must do `SELECT counter; UPDATE SET counter = $1+1`. |
| Embedding-UPDATE rejection | Cross-shard atomic moves are their own distributed-transactions problem. | Apps must `DELETE + INSERT` to change a vector. |
| Coordinator-only routing decisions | Coordinator owns routing centroids and placement directory. | Coordinator availability is hard requirement for writes. Multi-coordinator HA is Phase 12+. |
| Two relation-context loaders (SPI for diagnostic, catalog for hook) | Hook needs catalog-backed metadata to avoid SPI re-entrance; diagnostic surface uses SPI loader. | Two implementations; parity test guards drift. Cleanup possible once SPI re-entrance bound. |
| `recheck` callback unconditionally `true` | EvalPlanQual rerun isn't meaningful for a CustomScan that doesn't carry MVCC row identity. | SERIALIZABLE / EvalPlanQual won't get PG's normal serialization-failure guarantee on distributed reads. **H5 covers**. |
| Compat-shim columns in cleanup diagnostics | Operator monitoring queries that grep for `row_materialization_orphan_count` don't break. | Zero-valued column cruft in API surface until 0.2.x. |

## Suggested packet sequence

Recommend addressing in this order:

1. **30897 — Typed tuple transport (P1)**: design + composite-record
   endpoint + dual-write coordinator path. Biggest perf lever.
2. **30898 — Placement gate index lookup (P2)**: switch from
   seqscan to index lookup. Smallest fix, biggest planner-time
   win.
3. **30899 — Crash recovery runbook (H3)**: GID format fix + ops
   doc.
4. **30900 — Concurrency test matrix (H1, H2, H4, H11, H12)**:
   parallel-INSERT, concurrent-DELETE, statement-timeout-mid-INSERT,
   multi-row-INSERT, placement contention.
5. **30901 — Type round-trip coverage (H6, H7)**: schema-drift
   detection + per-type round-trip fixtures.
6. **30902 — Async INSERT dispatch (P9)**: bring INSERT 2PC to
   parity with read path async pattern.
7. **30903 — `max_prepared_transactions` preflight (H9)**:
   readiness check + error wrapping.
8. **30904 — EvalPlanQual / SERIALIZABLE limitations (H5)**:
   document and add fixtures.
9. **30905 — Catalog-backed relation context cache (P4)**:
   per-snapshot cache.
10. **30906 — Stage F cost-model calibration (P6)**: benchmark-
    derived constants.
11. **30907 — Negative-coverage fixtures (H8)**: classifier
    edge-case rejection.
12. **30908 — `custom_private` layout cleanup (P7, P8)**: native
    PG list nodes + stack-allocated PK bytes.

Items 1, 2, 3 should land before any AWS scale entry. Items 4-7
should land before Stage F benchmarks claim production-readiness
numbers. Items 8-12 are quality-of-life and can land in parallel.

## Reviewer focus

This packet is a planning artifact. There is no code change to
review here — the request is for the coder to triage the items
above into concrete follow-up packets with their own request.md
files.

The two highest-priority asks per the user:
- **Hardening (Theme 1)**: address H1-H12 with non-happy-path
  fixtures.
- **Performance (Theme 2)**: P1 (retire JSON bridge) is the
  single biggest lever. P2-P9 follow.

## References

Per-slice feedback files cross-referenced above. Highlights:

- 30810: executor stream — PK execution + recheck callback.
- 30814 / 30816 / 30880: tuple-payload JSON protocol.
- 30827: cost model symbolic constants.
- 30830: INSERT 2PC + GID format + sync dispatch.
- 30835 / 30836: trigger + descriptor refresh.
- 30839 / 30873: DELETE / PK SELECT primitives.
- 30855 / 30856: catalog-backed relation context.
- 30873: SPI placement probe → catalog scan transition.
- 30884: planner_hook plan-tree replacement.
- 30894: materialization AM cleanup + status string rename.
