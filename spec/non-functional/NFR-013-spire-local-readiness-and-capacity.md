---
id: NFR-013
title: SPIRE Local Readiness and Capacity
type: non-functional-requirement
artifact_type: NFR
status: APPROVED
relationships:
  - target: "ix://agent-ix/ecaz/FR-052"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-053"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-054"
    type: "constrains"
    cardinality: "1:N"
  - target: "ix://agent-ix/ecaz/FR-060"
    type: "constrains"
    cardinality: "1:N"
---
# NFR-013: SPIRE Local Readiness and Capacity

## Requirement

SPIRE local production-readiness smoke evidence SHALL use explicit bounded
fanout, payload, timeout, and concurrency settings and SHALL NOT be described
as AWS/RDS or product-scale evidence.

## Measurement Contract

Local readiness packets SHALL record:

- evidence label: `local production-readiness smoke`;
- node count, selected PID count, remote fanout, candidate counts, heap rows,
  route counts, object bytes, local-store counters, strict failures, degraded
  skips, timeout/cancel counts, and placement contention when available;
- typed tuple transport status and any compatibility fallback still reachable;
- active remote fanout, payload, timeout, and concurrency GUCs.

## Capacity Baseline

The local readiness profile SHALL use these default smoke boundaries unless a
review packet records a stricter or explicitly measured replacement:

| Surface | Target |
| --- | ---: |
| ready remotes per coordinator query | 8 |
| remote leaf PIDs per coordinator query | 256 |
| selected PIDs per remote node | 64 |
| tuple payload bytes per row | 1024 |
| tuple payload rows per batch | 64 |
| concurrent distributed-read coordinator sessions | 1 |
| concurrent remote-search dispatches across coordinator backends | 8 |
| concurrent remote-search dispatches per remote node | 1 |
| concurrent coordinator-routed writer workloads | 1 |
| concurrent work per remote node | 1 read dispatch or 1 prepared write branch |

## Claim Boundary

Local readiness evidence SHALL NOT claim product-scale capacity, managed-service
behavior, cross-AZ behavior, WAN behavior, AWS/RDS latency, AWS/RDS throughput,
or safe higher concurrency without packet-local measurements for the tested
fixture.

## Acceptance Criteria

### NFR-013-AC-1

Local readiness spec rows and review packets distinguish local functionality,
local production-readiness smoke, and AWS/RDS product-scale evidence.

### NFR-013-AC-2

Readiness artifacts record the GUCs and counters needed to interpret fanout,
payload, timeout, strict/degraded, and concurrency behavior.

### NFR-013-AC-3

Specs do not promote local smoke results into product-scale claims.
