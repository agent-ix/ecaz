# SPIRE Phase 11 Paper Parity and Production Gate

Status: active Phase 11.1 gate
Task: Task 30 Phase 11
Paper basis: `/home/peter/dev_bak/papers/2512.17264v1.pdf`

## Source Basis

The local paper artifact is titled "Scalable Distributed Vector Search via
Accuracy Preserving Index Construction" and its PDF outline names SPire design
sections for accuracy-preserving hierarchy, disaggregated index store,
stateless query execution, balanced partition granularity, parallel index
construction, deployment/robust operation, performance/scalability, level-count
impact, ablation, and extreme-scale simulation.

This document is the durable Phase 11.1 traceability artifact. It does not
claim SPire paper parity yet. It defines what must be true before the deferred
AWS/RDS-class scale packet can start.

## Traceability Matrix

| Paper section / mechanism | Current ecaz state | Phase 11 gate | Evidence owner | Status |
| --- | --- | --- | --- | --- |
| SPire design overview | Task 30 owns SPIRE as a partition-object IVF hierarchy, not a generic distributed SQL engine. | Keep one coherent production path from hierarchy to remote fanout to final rows. | Task 30 overview and Phase 11 packets | In progress |
| Accuracy-preserving hierarchy | Phase 9 closed local hierarchical routing, level budgets, and top-graph storage. | Distributed remote endpoints must obey the same selected PID, epoch, and route-budget contracts. | Phase 9 closeout and Phase 11.3 | Local-only ready |
| Top-level graph routing | Local top-graph chain storage and borrowed routing are landed. | Preserve as the coordinator entry path; add distributed route/fanout diagnostics without rebuilding graph ownership. | Phase 9 packets and Phase 11.6 | Local-only ready |
| Balanced partition granularity | Local nlist/nprobe controls and pipeline counters exist; AWS scale evidence is deferred. | Define local capacity targets and only schedule AWS after Phase 11.1-11.8 pass or are explicitly deferred. | Phase 11.8 and 11.9 | Open |
| Boundary replication | Local boundary-replica assignment and scan diagnostics exist. | Cross-node replicas must share global `0x02` vector IDs and dedupe once across nodes. | Phase 11.2 and 11.5 | Open |
| Stable vector identity | ADR-055 defines local `0x01` and global `0x02` identities; allocation hooks, the 16-byte stable source contract, fixed-width Leaf V2 global-ID storage, and ADR-063's included identity-column provider plan are landed, while live writers still default to local IDs. | Writers must emit global IDs from ADR-063's live source-identity provider; local-only indexes need explicit compatibility diagnostics. | Phase 11.2 | Open |
| Disaggregated index store | Local placement maps already model `(node_id, local_store_id)` and local multi-store object placement. | Multi-instance manifests must publish remote placement readiness and replica freshness without AWS. | Phase 11.6 and 11.7 | Open |
| Stateless query execution engine | SQL-visible libpq surfaces and receive/merge contracts exist, but ADR-058 keeps the executor diagnostic-only. | Production AM remote fanout must be concurrent or pipelined, bounded, cancellable, timed out, and observable. | Phase 11.4 | Open |
| Remote near-data scoring | Diagnostic remote search candidates exist. | Production endpoint must score at origin, return compact candidates, bind served epoch plus quantizer/index fingerprint, and reject incompatible remotes in strict mode. | Phase 11.3 | Open |
| Remote heap/final row delivery | ADR-059 keeps remote row locators opaque and blocks production remote final rows. | Origin-node heap visibility filtering must return final rows before the coordinator claims SQL row readiness. | Phase 11.5 | Open |
| Quantized scoring | RaBitQ is the supported first SPIRE quantized path. | Keep RaBitQ in scope and bind its profile/fingerprint into remote candidate compatibility. | Phase 11.3 and 11.8 | Open |
| PQ/PQFastScan | Reserved/unsupported for SPIRE today. | Explicitly exclude from Phase 11 parity claims unless a later task lands support. | Phase 11.1 and future task | Deferred |
| Parallel index construction | Local build/update publication is relation-backed and epoch-based. | Remote manifest publication, online lifecycle behavior, and version skew must be defined before production distribution. | Phase 11.6 | Open |
| Deployment and robust operation | Local diagnostics are broad; remote connection security, resource governance, and fault injection need production gates. | Preserve libpq `sslmode` through secret resolution, keep raw conninfo hidden, define sanitized strict/degraded auth/cert failure behavior, add global/per-remote backpressure, and add strict/degraded fault matrix. | Phase 11.4, 11.6, 11.8 | Open |
| Performance/scalability evidence | Local 10k and pipeline counter evidence exists; product scale is not claimed. | Local multi-instance recall/latency/counter bundle must pass before AWS is scheduled. | Phase 11.8 and 11.9 | Open |
| Extreme-scale simulation | Not implemented as a product claim. | Keep out of Phase 11 exit unless implemented as a non-product planning model with clear labels. | Future scale packet | Deferred |

## Diagnostic vs Production Surfaces

The following surfaces are useful for review packets and operators but are not
the production distributed AM path until their Phase 11 production counterparts
land:

| Surface family | Current role | Production requirement |
| --- | --- | --- |
| `ec_spire_remote_search_libpq_*` SQL functions | Diagnostic executor planning, binding, connection, dispatch, receive, and summary probes. | Production coordinator executor in the AM scan path with bounded concurrent or pipelined remote work. |
| `ec_spire_remote_search_*_summary` gate functions | Packet-friendly status aggregation. | Must remain aligned with production executor state and failure reasons. |
| `ec_spire_remote_search_coordinator_result_summary` | Final result readiness proof and blocker reporting. | Must only report remote SQL-row readiness after origin-node heap resolution. |
| `ec_spire_remote_epoch_manifest_*` SQL functions | Remote manifest planning and diagnostic publication contract. | Production publication path must validate descriptor, epoch, manifest freshness, and online lifecycle behavior. |
| `ecaz bench spire-pipeline` | Local and simulated counter capture. | Distributed sibling or extension must capture local multi-instance recall, latency, fanout, timeout, cancel, fault, and degraded counts. |

## Pre-AWS Production Gate

AWS/RDS-class scale remains blocked until all of these local gates pass or have
explicit accepted deferrals:

1. Paper traceability rows above are either ready, local-only ready with an
   accepted production follow-up, or explicitly deferred.
2. Writer-side global vector IDs are available from a live 16-byte stable
   source-identity provider, and local-only IDs remain node-scoped with visible
   diagnostics.
3. Remote search endpoint returns compact, validated candidates bound to served
   epoch, node identity, vector identity, row locator, score, flags,
   quantizer/index fingerprint, extension/protocol version, and diagnostics.
4. Production libpq coordinator fanout is concurrent or pipelined, bounded,
   timed out, cancellable, and covered by strict/degraded failure semantics.
5. Remote heap resolution is origin-node owned and tested; the coordinator does
   not decode remote row locators into local heap TIDs.
6. Local multi-instance fixture covers one coordinator plus at least two remote
   PostgreSQL nodes, including epoch mismatch, version skew, stale remote,
   dropped/reindexed remote index, slow remote, cancellation, timeout,
   connection reset, backend termination, simulated partition, and remote OOM.
7. Remote connection security preserves libpq `sslmode` from the
   `conninfo_secret_name` provider without stripping or downgrading it; strict
   mode rejects libpq authentication or certificate-verification failures with a
   sanitized error, degraded mode reports the remote as skipped, and raw
   conninfo remains unexposed from SQL.
8. Resource governance runbook states local targets for maximum remotes,
   concurrent coordinator queries, concurrent work per remote, PIDs per node,
   and overload/degraded behavior.
9. Local multi-store/multi-NVMe harness reports per `(node_id, local_store_id)`
   route counts, candidate counts, object bytes, and read scheduling limits.
10. Packet-local artifacts prove recall, latency p50/p95/p99, remote fanout,
    heap rows, timeout/cancel counts, strict failure counts, and degraded skip
    counts from a clean local setup.

## Explicit Deferrals

- AWS/RDS-class product-scale measurement.
- Billion-scale or trillion-scale claims.
- PQ/PQFastScan SPIRE scoring.
- Distributed writes and cross-node read-after-write semantics.
- Coordinator HA, coordinator election, and multi-coordinator consensus.
- A custom network protocol beyond libpq/pipeline mode.
- Credential rotation, audit-log schema, and a full TLS runbook beyond the
  Phase 11 libpq `sslmode` preservation and sanitized failure contract.

## First Implementation Slice

Phase 11.2 writer-side global vector identity is the first code slice because
remote endpoint promotion, production merge, boundary-replica correctness, and
origin-node heap finalization all depend on stable cross-node identity. The
allocation hook, 16-byte source-identity contract, and Leaf V2 fixed-width
`GlobalBytes` storage layout are now available. ADR-063 selects the v1 provider
as one included UUID or exact-16-byte `bytea` identity column; the remaining
Phase 11.2 gate is to implement that provider in live build/insert writers.
