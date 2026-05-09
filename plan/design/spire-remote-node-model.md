# SPIRE Remote Node Model

Status: Phase 7 design checkpoint for Task 30
Date: 2026-05-07
Scope: remote node identity, placement membership, health, and stale-node
behavior before distributed libpq search implementation

This note defines the first multi-machine placement model for SPIRE. It is
subordinate to ADR-049, the Phase 0 partition-object storage design, the Phase
4 local multi-store placement design, FR-041 epoch consistency, and FR-042
distributed libpq coordination.

## Goals

Phase 7 extends the existing logical placement map from local stores:

```text
pid -> node_id -> local_store_id -> object location
```

The first remote checkpoint must settle:

- what a `node_id` means and how it differs from a PostgreSQL OID, hostname, or
  connection string;
- how remote placement entries become eligible for scan;
- which node states participate in reads and writes;
- how the coordinator detects stale or unavailable nodes;
- how strict and degraded modes behave before remote libpq execution code
  exists.

This phase does not introduce replicated partition objects, cross-node
consensus, automatic rebalancing, remote DDL orchestration, or product-scale
availability claims.

## Node Identity

`node_id` is a coordinator-assigned SPIRE identifier scoped to one coordinator
index OID. It is not a PostgreSQL relation OID, a remote server OID, a host
name, a DSN, or a Kubernetes pod identity.

Reserved values:

```text
node_id = 0     local coordinator node
node_id > 0     remote SPIRE storage node
```

The coordinator owns the `node_id` namespace. A remote node must echo the
expected `node_id` and remote index identity during capability and health
checks before the coordinator can mark that node eligible for a distributed
epoch.

Connection details are mutable metadata attached to a stable node ID. Rotating
credentials, changing hostnames, or replacing a remote PostgreSQL instance must
not silently change the meaning of an existing `node_id`. If the replacement
cannot prove it serves the same remote index identity and retained epoch window,
the coordinator must treat it as a new node or require an explicit repair
operation.

## Remote Node Descriptor

The first durable descriptor should carry:

```text
spire_remote_node
  coordinator_index_oid oid
  node_id u32
  generation u64
  conninfo_secret_name text
  remote_index_identity bytea
  remote_index_regclass text | oid
  state active | draining | disabled | failed
  last_seen_at timestamptz
  last_served_epoch u64
  min_retained_epoch u64
  extension_version text
  last_error text
```

`conninfo_secret_name` is intentionally indirect. Raw connection strings should
not become part of SPIRE epoch manifests or review artifacts.

`generation` changes when the node membership set changes. A distributed
published epoch references exactly one remote-node generation, just as Phase 4
local store placement references one local-store generation. Changing
membership is a rebuild, repair, or explicit rebalance boundary; it is not an
incidental scan-time side effect.

## Node States

Remote node state is coordinator policy, not just network reachability.

```text
active
  eligible for new placements, reads, and remote searches

draining
  eligible to serve retained epochs and active placements, but not eligible for
  new placements in future epochs

disabled
  intentionally excluded from reads and writes; strict epochs requiring it fail
  and degraded epochs may skip it only with diagnostics

failed
  last health check or remote search failed; strict epochs requiring it fail and
  degraded epochs may skip it only with diagnostics
```

`active` and `draining` nodes may serve reads if they can prove the requested
epoch and object versions are retained. `disabled` and `failed` nodes are never
used for candidate production.

The coordinator must not convert `failed` back to `active` purely because a TCP
connection succeeds. Recovery requires a capability check that validates node
identity, extension compatibility, remote index identity, and served epoch
range.

## Placement Membership

The existing placement entry remains the logical compatibility boundary:

```text
placement_entry
  epoch
  pid
  node_id
  local_store_id
  store_relid
  object_version
  object_tid
  object_bytes
  state available | stale | unavailable | skipped
```

For remote placements:

- `node_id` must be nonzero and present in the epoch's node generation.
- `local_store_id` is interpreted on the remote node, not on the coordinator.
- `store_relid` is a remote diagnostic locator. It must not be opened by the
  coordinator as a local relation.
- `object_tid` and `object_bytes` describe the remote node's object location
  and payload size for diagnostics and remote validation.
- v1 publishes one primary placement for each PID. Replicated partition objects
  remain deferred.

A distributed epoch is publishable only after every `Available` remote
placement has write evidence from the owning node for the exact `(pid,
object_version)` being referenced.

## Health and Staleness

The coordinator determines node eligibility with a remote capability check that
returns at least:

```text
node_id
remote_index_identity
extension_version
state
last_served_epoch
min_retained_epoch
supported_candidate_format
```

A node is stale for requested epoch `E` when:

- it cannot prove `last_served_epoch >= E`;
- it has already dropped `E` from its retained epoch window;
- it reports a different `remote_index_identity`;
- it reports an incompatible extension or candidate format;
- its descriptor generation does not match the epoch generation.

Stale nodes are not readable in v1. If an epoch was published as degraded, the
coordinator may mark affected placements `Skipped` or `Unavailable` and
continue only when diagnostics report the skipped `(node_id, pid)` pairs. A
placement in `Stale` state remains a diagnostic fact, not a candidate source.

## Strict and Degraded Reads

Strict mode:

- every selected local and remote placement must be `Available`;
- every required remote node must be `active` or `draining`;
- every required remote node must prove it can serve the requested epoch;
- any unavailable, disabled, failed, stale, or identity-mismatched node causes
  the query to fail closed.

Degraded mode:

- selected placements on unavailable or failed nodes may be skipped only when
  the active epoch was explicitly published as degraded;
- skipped nodes and PIDs must be visible in diagnostics for the served query;
- stale placements are not read;
- candidate merge must carry a degraded-result marker so operators can
  distinguish lower recall from normal empty results.

Local strict behavior remains the default. Distributed degraded behavior is an
explicit operating mode, not a silent fallback from strict.

## Candidate Identity

Remote candidates must carry enough identity for coordinator-side merge and
final row delivery:

```text
remote_candidate
  served_epoch
  node_id
  pid
  vec_id
  row_locator
  score
  assignment_flags
```

`vec_id` is the dedupe and boundary-replica merge key. Local Phase 1 `vec_id`
values can participate in a distributed test fixture only when they are unique
across all participating nodes by construction. Production distributed SPIRE
requires the reserved global `vec_id` encoding from the Phase 0 design before
cross-node candidate merge can be claimed durable. Until that global encoding
lands, candidate merge by raw `vec_id` bytes is a production blocker for
multi-node fanout and is safe only for a single node or a fixture that proves
global uniqueness outside SPIRE.

Likewise, until retained-epoch serving lands, the only valid requested epoch
for remote fanout is the coordinator's published active epoch. A node that can
serve an older retained epoch is a future capability; v1 active fanout must
fail closed on any requested epoch mismatch.

`row_locator` is opaque to the coordinator until a row-delivery design lands.
It may encode a remote heap TID plus relation identity, but the coordinator
must not treat it as a local heap TID. ADR-059 assigns production remote heap
resolution to the origin node and keeps remote final rows blocked until
origin-node heap visibility checks and global vector IDs land.

## Publish Boundaries

Distributed publication keeps the existing SPIRE rule: objects are immutable,
and visibility changes only through epoch manifests.

The coordinator may publish a distributed epoch only after:

1. every target remote node has accepted and durably stored its assigned
   objects;
2. every target remote node can report the accepted `(pid, object_version)`
   evidence;
3. the coordinator has built an epoch manifest and placement directory using
   one remote-node generation;
4. strict/degraded consistency mode has been chosen before root/control is
   advanced.

A partial remote publish must remain invisible to active scans. Remote orphan
objects are cleanup candidates on their owning nodes, not active epoch members.

## Diagnostics

Phase 7 should add diagnostics before enabling remote query execution:

- node descriptor snapshot: state, generation, last seen, served epoch range,
  extension version, last error;
- placement membership snapshot by `(node_id, local_store_id)`;
- remote stale/unavailable reason rows by `(node_id, pid)`;
- per-query degraded summary: requested epoch, skipped node count, skipped PID
  count, and whether recall may be reduced.

The existing local placement diagnostics already group by `(node_id,
local_store_id)`. Remote diagnostics should extend that surface instead of
creating a separate placement vocabulary.

## Deferred

- replicated partition objects for read throughput or availability;
- automatic remote node discovery;
- remote DDL and schema migration orchestration;
- cross-node object rebalancing;
- global `vec_id` rewrite implementation;
- remote row fetch and final row delivery mechanics;
- production AM remote libpq execution. ADR-058 keeps the current SQL-visible
  libpq executor diagnostic/operator-only until pipeline or async fanout,
  cancellation, timeout, fail-closed, and final row delivery semantics land.
