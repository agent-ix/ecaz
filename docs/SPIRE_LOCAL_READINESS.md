# SPIRE Local Readiness and AWS Evidence Boundaries

This document defines the evidence labels used by Task 30 Phase 12 and Phase
13. The labels are claim boundaries: use the narrowest label supported by the
artifact packet.

## Local Functionality

Local functionality evidence proves that a feature works in a focused local
fixture. It can come from Rust unit tests, PG18 pgrx tests, SQL diagnostic
queries, or local one-index fixtures.

Local functionality evidence may claim:

- the tested code path works for the covered fixture;
- the SQL/operator surface returns the documented fields and labels;
- the fixture covers the named edge case or regression.

Local functionality evidence may not claim:

- production readiness for distributed operation;
- capacity targets;
- AWS/RDS behavior;
- latency or throughput improvements unless the packet includes a benchmark
  harness and raw logs.

## Local Production-Readiness Smoke

Local production-readiness smoke evidence proves that the local distributed
SPIRE path can be set up cleanly and exercised across the Phase 12 hardening
surface before AWS verification starts.

A local production-readiness smoke packet should include:

- clean setup and teardown from repo-owned commands or documented scripts;
- distributed read and write fixtures against the current CustomScan path;
- strict and degraded fault checks, including skipped/stale node reporting;
- typed tuple transport status and any JSON compatibility fallback still
  reachable;
- 2PC readiness, cancellation/failure behavior, and orphaned prepared xact
  operator guidance;
- placement, local-store, and boundary-replica diagnostics;
- local counters for recall, latency, object bytes, route counts, candidate
  counts, heap rows, fanout, timeout/cancel counts, strict failures, degraded
  skips, and placement contention when those harnesses exist.

Local production-readiness smoke evidence may claim:

- the local distributed implementation is ready for Phase 13 AWS/RDS-class
  verification when all required Phase 12 rows are complete or explicitly
  reviewer-deferred;
- locally measured capacity targets for the tested machine and fixture.

Local production-readiness smoke evidence may not claim:

- product-scale capacity;
- managed-service behavior;
- cross-AZ/network behavior;
- AWS/RDS latency, throughput, or reliability.

## AWS/RDS Product-Scale Evidence

AWS/RDS product-scale evidence belongs to Task 30 Phase 13. It requires a
packet-local manifest that pins topology, instance classes, storage, region/AZ
layout, dataset, query mix, run commands, and raw logs before product-scale
claims are made.

AWS/RDS evidence may claim only what the Phase 13 packet directly measures:

- cloud topology and managed-service compatibility;
- correctness under the selected dataset and query/write mix;
- latency, throughput, recall, and capacity targets for the tested AWS/RDS
  configuration;
- operational behavior for the tested failure, timeout, cancellation, and
  credential scenarios.

Phase 13 must not start by implementing missing Phase 12 hardening. Any
accepted Phase 12 deferral must be repeated in the AWS report so product-scale
claims do not hide local readiness gaps.

## Claim Rules

- Cite packet-local artifacts for every measurement claim.
- Name the evidence label in review requests and runbooks.
- Do not translate local functionality into readiness without the local smoke
  bundle.
- Do not translate local smoke into AWS/RDS product-scale evidence.
- Keep AWS/RDS work blocked until Phase 12 exit criteria are complete or
  explicitly reviewer-deferred.
