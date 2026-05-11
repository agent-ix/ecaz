---
id: ADR-066
title: "SPIRE Operator-Owned Row Materialization Mirror Sync"
status: PROPOSED
impact: Defines the Task 30 Phase 11 Stage D mechanism that populates remote row materialization state outside AM scans
date: 2026-05-10
---
# ADR-066: SPIRE Operator-Owned Row Materialization Mirror Sync

## Status

Proposed.

## Related

- ADR-064 requires pre-existing same-relation coordinator heap rows before the
  SPIRE index AM may return remote-origin candidates.
- ADR-065 defines the extension-owned mapping catalog that links remote
  candidate identity to coordinator heap TIDs.
- Packet 30799 proves that explicit catalog registration can make a
  remote-origin candidate return through PostgreSQL `amrescan` / `amgettuple`.

## Context

The AM now has a read-only catalog provider and an end-to-end SQL proof with an
explicit `ec_spire_register_remote_row_materialization(...)` call. That is not
enough for production: a real deployment still needs a mechanism that creates
the coordinator heap rows and registers catalog mappings before user scans run.

The mechanism must respect these constraints:

- `amrescan` / `amgettuple` cannot write user heap rows or catalog mappings.
- The returned TID must belong to the same heap relation that owns the SPIRE
  index.
- The coordinator must not decode origin heap TIDs from `row_locator`; origin
  row identity remains remote-owned and opaque.
- Generic row mirroring cannot be inferred from a vector index alone. It needs
  an operator-declared relation identity, selected source columns, conflict
  policy, and write target.

## Decision

SPIRE v1 uses an explicit operator-owned mirror refresh mechanism, exposed as a
SQL primitive and wrapped by `ecaz`, to populate coordinator materialization
state outside index scans.

The v1 production mechanism is:

1. An operator declares a mirror profile for a coordinator SPIRE index. The
   profile names the remote node descriptor, remote relation or query source,
   stable source identity column, target coordinator heap relation, target
   insert/update column mapping, conflict policy, and epoch window.
2. The operator runs a refresh command after remote endpoint readiness and
   before advertising an epoch as AM-deliverable.
3. The refresh command fetches remote rows through the configured remote source,
   inserts or updates ordinary coordinator heap rows in the indexed relation,
   obtains their coordinator TIDs, and calls the catalog registration path with
   requested epoch, served epoch, origin node, global `vec_id`, opaque
   `row_locator`, and materialized coordinator TID.
4. The AM scan path only reads the catalog and validates heap visibility under
   the scan snapshot.

The first implementation should be deliberately narrow:

- require explicit operator invocation;
- require an explicit stable identity column or expression already accepted by
  the SPIRE global identity contract;
- support one coordinator index and one remote node per call;
- use strict mode by default, failing the refresh if endpoint identity,
  extension version, epoch, row shape, or heap registration does not validate;
- expose degraded/dry-run diagnostics before broad multi-node automation.

`ecaz` should become the operator entrypoint once the SQL primitive exists, so
repeatable local and production runs can be logged into review packets and
runbooks without ad hoc SQL scripts.

## Rejected Alternatives

### Background Worker First

A background worker would hide the lifecycle from the operator and force early
answers for scheduling, retry, credential refresh, backpressure, and crash
recovery. Those are production concerns, but they should be layered after the
explicit refresh path is correct and observable.

### Lazy Per-Query Materialization

Lazy materialization during a user query violates the ADR-064 scan-time write
rule if it happens inside or on behalf of `amrescan` / `amgettuple`. Running a
separate transaction immediately before a query would also make snapshot
semantics hard to explain and would surprise read-only query paths.

### Generic Logical Replication First

Logical replication may eventually maintain coordinator mirrors, but it does
not by itself register SPIRE `vec_id` / `row_locator` mappings or bind them to
served epochs and endpoint identity. It also adds cluster setup, slot, and
replication lifecycle concerns before the materialization contract is fully
tested.

### User-Managed Register Calls Only

Direct calls to `ec_spire_register_remote_row_materialization(...)` are useful
for tests and emergency repair, but they are too low-level to be the production
mechanism. They do not create heap rows, do not fetch remote origin state, do
not enforce row-shape policy, and are easy to misuse.

## Required Implementation Slices

1. Add a mirror profile contract and dry-run diagnostic surface for one
   coordinator index plus one remote node.
2. Add the refresh SQL primitive that materializes rows and registers mappings
   outside the AM scan path.
3. Wrap the primitive with an `ecaz` operator command that writes packet-local
   logs.
4. Add a PG18 fixture proving: epoch readiness -> refresh command -> catalog
   populated -> SQL `SELECT` returns remote rows without explicit register calls
   in the test body.
5. Add lifecycle coverage for stale coordinator rows, in-flight scans,
   concurrent refresh, epoch retirement cleanup, catalog bounds, and mirror
   repair after vacuum/deletion.

## Invariants

- Mirror refresh is a maintenance operation, not an AM callback.
- Refresh must be idempotent for the same epoch and stable remote identity.
- Refresh must not expose raw libpq conninfo in SQL rows, logs, or errors.
- Refresh must preserve endpoint identity validation before accepting remote
  rows.
- Mapping registration must remain exact: index, requested epoch, served epoch,
  origin node, global vector identity, opaque row locator, scan heap relation,
  and coordinator TID.
- AM delivery remains fail-closed when a catalog row is missing, stale, or not
  visible under the scan snapshot.

## Consequences

- Stage D remains open until the refresh primitive, `ecaz` wrapper, and
  no-explicit-register PG18 fixture land.
- The first production-ready path is operationally explicit but observable and
  testable.
- Later automation can promote the explicit refresh into a background worker,
  logical replication integration, or epoch-publish hook without changing the
  AM scan contract.
