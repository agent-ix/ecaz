---
id: FR-060
title: SPIRE Diagnostics Configuration and Operator Surface
type: functional-requirement
artifact_type: FR
status: APPROVED
object: api_endpoint
relationships:
  - target: "ix://agent-ix/ecaz/FR-048"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-057"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-059"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-060: SPIRE Diagnostics Configuration and Operator Surface

## Description

SPIRE SHALL expose read-only SQL diagnostics, bounded configuration, and
operator command surfaces that explain active index state, routing behavior,
remote readiness, DML/recovery posture, and evidence labels without exposing
raw secrets or making performance claims by inspection alone.

## Diagnostic Groups

```mermaid
flowchart LR
    Op["operator / review packet"]
    SQL["read-only SQL diagnostics"]
    CLI["ecaz SPIRE workflows"]
    IDX["index metadata\nobjects + epochs + options"]
    REM["remote executor\ntransport + skips + faults"]
    DML["DML / 2PC recovery\nclassifier + prepared xacts"]
    Evidence["packet-local evidence\nlabels + counters + logs"]

    Op --> SQL
    Op --> CLI
    SQL --> IDX
    SQL --> REM
    SQL --> DML
    CLI --> SQL
    CLI --> Evidence
    IDX --> Evidence
    REM --> Evidence
    DML --> Evidence
```

| Group | Examples | Purpose |
| --- | --- | --- |
| Health and active state | `ec_spire_index_health_snapshot`, `ec_spire_index_active_snapshot_diagnostics` | One-row state, epoch, object, placement, and byte-count overview. |
| Storage and epoch cleanup | `ec_spire_index_relation_storage_snapshot`, `ec_spire_index_epoch_cleanup_summary`, `ec_spire_index_epoch_cleanup_run` | Old epoch retention, cleanup debt, and safe reclamation. |
| Routing and scan | `ec_spire_index_scan_routing_snapshot`, `ec_spire_index_scan_placement_snapshot`, `ec_spire_index_scan_local_store_execution_snapshot` | Route budgets, selected PIDs, local store grouping, candidate counts. |
| Candidate attribution | funnel, pipeline, target-block-rank, row-segment, and block-summary diagnostics | Distinguish routing, selected-block containment, candidate-budget, row-decode, heap-rerank, and approximate-scoring bottlenecks. |
| Boundary replica | `ec_spire_index_boundary_replica_identity_snapshot`, `ec_spire_index_boundary_replica_placement_diagnostics` | Replica identity and placement health. |
| Remote executor | `ec_spire_remote_search_production_executor_state_summary`, `ec_spire_remote_search_degraded_skip_report`, `ec_spire_remote_pipeline_steps` | Dry and live remote readiness, strict/degraded status, pipeline stages. |
| DML and recovery | `ec_spire_dml_frontdoor_*`, `ec_spire_reap_orphaned_remote_prepared_xacts` | DML classifier, primitive plans, 2PC recovery. |
| Cost and options | `ec_spire_index_options_snapshot`, `ec_spire_index_cost_tuning_snapshot` | Effective reloptions, GUCs, payload scannability, cost constants. |

## Configuration Contract

SPIRE configuration SHALL include bounded reloptions and GUCs for:

- `nlists`, `nprobe`, recursive fanout, top-graph controls, route budgets, and
  candidate limits;
- local store count and local store tablespaces;
- source identity provider and boundary replica count;
- remote consistency mode, remote node/PID fanout limits, payload byte caps,
  connect/statement timeouts, and advisory governance limits;
- planner cost constants and storage/rerank multipliers.

Defaults MAY be permissive for development, but local readiness packets SHALL
record explicit nonzero fanout, concurrency, timeout, and payload caps.

## Stable Labels

Diagnostic status labels SHALL be treated as operator-facing contracts. A new
meaning SHALL use a new label rather than reusing an existing label.

Superseded materialization labels SHALL NOT be used for current distributed
CustomScan behavior. Current distributed read blockers use CustomScan tuple
delivery, typed-transport, endpoint identity, schema drift, budget/governance,
timeout, or degraded-skip labels.

## Operator CLI And Evidence

The `ecaz` CLI SHALL own repeatable SPIRE operator workflows where a shell
script or SQL sequence becomes part of review evidence. Current surfaces include
SPIRE pipeline counters and `ecaz dev spire-multicluster` wrappers for smoke,
CustomScan read, insert-read-after-CustomScan, transport overlap, fault, and
lifecycle fixtures.

Measurement and readiness claims SHALL cite packet-local artifacts and one of
the evidence labels defined by the SPIRE readiness docs.

Product-scale Pareto or recall-recovery claims SHALL cite suite-driven
artifacts and candidate-surface diagnostics. Diagnostics that show a larger
candidate surface, wider top-graph search, or wider block cap improved recall
SHALL be reported as a tradeoff unless latency and candidate gates also pass.

## Endpoint

SPIRE's operator endpoints are SQL diagnostic functions on the coordinator (and
locally on each node), not HTTP routes. `Method` is the SQL invocation shape,
`Path` is the function identity, and `Auth` is the PostgreSQL privilege needed.

| Method | Path | Auth | Description |
| --- | --- | --- | --- |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_health_snapshot` | table owner or `pg_monitor`-style read role | One-row state, epoch, object, placement, and byte-count overview. |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_active_snapshot_diagnostics` | read role | Active epoch object/placement detail. |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_relation_storage_snapshot`, `ec_spire_index_epoch_cleanup_summary` | read role | Storage layout, old-epoch retention, and cleanup debt. |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_epoch_cleanup_run` | index owner (mutating) | Safe reclamation of superseded epoch objects. |
| `SELECT * FROM fn(regclass, ...)` | `ec_spire_index_scan_routing_snapshot`, `ec_spire_index_scan_placement_snapshot`, `ec_spire_index_scan_local_store_execution_snapshot` | read role | Route budgets, selected PIDs, local store grouping, candidate counts. |
| `SELECT * FROM fn(regclass, ...)` | candidate-attribution funnel, pipeline, target-block-rank, row-segment, and block-summary diagnostics | read role | Miss attribution across routing, selected-block containment, candidate budget, row decode, heap rerank, and approximate scoring. |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_boundary_replica_identity_snapshot`, `ec_spire_index_boundary_replica_placement_diagnostics` | read role | Boundary replica identity and placement health. |
| `SELECT * FROM fn(...)` | `ec_spire_remote_search_production_executor_state_summary`, `ec_spire_remote_search_degraded_skip_report`, `ec_spire_remote_pipeline_steps` | read role; live variants open libpq/TLS sockets | Dry and live remote readiness, strict/degraded status, pipeline stages. |
| `SELECT * FROM fn(...)` | `ec_spire_dml_frontdoor_*` | write role for plan execution | DML classifier and primitive plans. |
| `SELECT fn(...)` | `ec_spire_reap_orphaned_remote_prepared_xacts` | superuser/operator role (mutating) | Two-phase-commit orphan recovery. |
| `SELECT * FROM fn(regclass)` | `ec_spire_index_options_snapshot`, `ec_spire_index_cost_tuning_snapshot` | read role | Effective reloptions, GUCs, payload scannability, cost constants. |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-060-AC-1 | An operator can inspect active epoch, object, placement, scan, route, local store, remote, DML, cost, and cleanup state via SQL diagnostics | Demonstration |
| FR-060-AC-2 | Diagnostics do not expose raw conninfo secrets or raw remote error text | Test |
| FR-060-AC-3 | Stable labels and evidence labels are documented as public contracts and stale row-materialization labels are not used for the current read path | Inspection |
| FR-060-AC-4 | Effective SPIRE reloptions and GUCs can be inspected in enough detail to reproduce routing, fanout, timeout, payload, and degraded-mode behavior | Demonstration |
| FR-060-AC-5 | Remote executor diagnostics expose endpoint identity, transport readiness, timeout/cancel state, strict failures, degraded skips, and payload compatibility without secrets | Test |
| FR-060-AC-6 | DML diagnostics expose classifier outcome, primitive plan inputs, placement directory state, prepared transaction intent state, and operator recovery actions | Test |
| FR-060-AC-7 | Repeatable SPIRE operator workflows can write packet-local logs and cite stable evidence labels for readiness, read, fault, and DML fixtures | Demonstration |
| FR-060-AC-8 | Diagnostics distinguish implementation readiness from benchmark claims and do not imply product-scale performance without packet-local measurement artifacts | Inspection |
| FR-060-AC-9 | Recall/latency diagnostics expose candidate-surface and miss-attribution fields that support measurement decisions without terminal scrollback or scratch files | Analysis |

### FR-060-AC-1: SQL state inspection

An operator can inspect active epoch, object, placement, scan, route, local
store, remote, DML, cost, and cleanup state through SQL diagnostics.

### FR-060-AC-2: Secret hygiene

Diagnostics do not expose raw conninfo secrets or raw remote error text.

### FR-060-AC-3: Label contracts

Stable labels and evidence labels are documented as public contracts and stale
row-materialization labels are not used as the current distributed read path.

### FR-060-AC-4: Configuration inspectability

Effective SPIRE reloptions and GUCs can be inspected with enough detail to
reproduce routing, fanout, timeout, payload, and degraded-mode behavior.

### FR-060-AC-5: Remote executor observability

Remote executor diagnostics expose endpoint identity, transport readiness,
timeout/cancel state, strict failures, degraded skips, and tuple-payload
compatibility without exposing raw secrets.

### FR-060-AC-6: DML and recovery observability

DML diagnostics expose classifier outcome, primitive plan inputs, placement
directory state, prepared transaction intent state, and operator-owned recovery
actions.

### FR-060-AC-7: Repeatable evidence workflows

Repeatable SPIRE operator workflows can write packet-local logs and cite stable
evidence labels for local readiness, distributed read, transport fault, and DML
lifecycle fixtures.

### FR-060-AC-8: Readiness vs claims

Diagnostics distinguish implementation readiness from benchmark claims and do
not imply product-scale performance without packet-local measurement artifacts.

### FR-060-AC-9: Miss-attribution fields

SPIRE recall/latency diagnostics expose candidate-surface and miss-attribution
fields that can support Task 73-85 style decisions without relying on terminal
scrollback or external scratch files.

## Dependencies

- **Upstream**: FR-048 (domain model: epochs, objects, placements the
  diagnostics describe), FR-057 (remote executor states and degraded-skip
  reporting surfaced here), FR-059 (DML classifier, prepared-xact intents,
  and reaper surfaced here).
- **Downstream**: none identified.
