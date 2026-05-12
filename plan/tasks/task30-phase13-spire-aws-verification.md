# Task 30 Phase 13: SPIRE AWS Verification

Status: blocked on Phase 12 production hardening
Owner: coder1 / SPIRE AWS verification track
Priority: after Phase 12 exit criteria are met

## Goal

Run AWS/RDS-class verification only after the local CustomScan distributed path
has passed Phase 12 hardening. Phase 13 is the external-scale evidence phase,
not a place to discover known local hardening gaps.

## Entry Gate

- [ ] Phase 12 is complete or every remaining Phase 12 item has an accepted
  reviewer deferral.
- [ ] The final local production-readiness bundle passes from clean setup.
- [ ] Operator runbook covers typed tuple transport, 2PC recovery,
  `max_prepared_transactions`, strict/degraded behavior, local capacity
  targets, and known v1 limitations.
- [ ] AWS packet manifest is prepared before allocating or running external
  infrastructure.

## Non-Goals

- Implementing missing Phase 12 hardening after AWS has already started.
- Claiming billion-scale product readiness from a partial local-readiness
  bundle.
- Reopening multi-coordinator HA, cross-shard non-vector query execution, DDL
  propagation, or cross-shard embedding UPDATE moves unless separate ADRs have
  already accepted that scope.

## Verification Plan

- [ ] Define the AWS/RDS topology:
  - [ ] coordinator instance class and storage;
  - [ ] remote instance classes and storage;
  - [ ] network placement;
  - [ ] PostgreSQL version and extension build;
  - [ ] security boundary and conninfo-secret setup.
- [ ] Define datasets:
  - [ ] small correctness dataset;
  - [ ] representative local-to-AWS scale dataset;
  - [ ] optional larger stress dataset only after smaller gates pass.
- [ ] Define workload matrix:
  - [ ] vector ORDER BY LIMIT reads through CustomScan;
  - [ ] coordinator-routed INSERT;
  - [ ] non-embedding UPDATE;
  - [ ] DELETE;
  - [ ] PK SELECT;
  - [ ] degraded-mode read behavior;
  - [ ] strict-mode fail-closed behavior.
- [ ] Capture correctness evidence:
  - [ ] remote rows returned through `EcSpireDistributedScan`;
  - [ ] placement directory matches remote heap ownership;
  - [ ] no materialization catalog/register calls;
  - [ ] source identity and boundary-replica dedupe remain stable;
  - [ ] Stage E fault/lifecycle smoke subset passes in AWS topology.
- [ ] Capture performance evidence:
  - [ ] recall;
  - [ ] latency p50/p95/p99;
  - [ ] throughput;
  - [ ] remote fanout;
  - [ ] tuple transport bytes and CPU counters;
  - [ ] route counts;
  - [ ] candidate counts;
  - [ ] heap rows;
  - [ ] object bytes;
  - [ ] timeout/cancel counts;
  - [ ] strict failure and degraded skip counts;
  - [ ] placement write-contention counters.
- [ ] Capture operations evidence:
  - [ ] clean setup and teardown transcript;
  - [ ] extension install/upgrade transcript;
  - [ ] runbook recovery drill for a synthetic orphaned prepared transaction,
    unless explicitly deferred;
  - [ ] required GUC verification on every remote;
  - [ ] sanitized error behavior for auth/certificate/conninfo failures.
- [ ] Publish packet-local artifacts for every AWS run:
  - [ ] commands used;
  - [ ] environment and instance metadata;
  - [ ] git SHA and extension version;
  - [ ] dataset identity;
  - [ ] raw logs;
  - [ ] summarized result table;
  - [ ] known caveats and accepted deferrals.

## Exit Criteria

- AWS verification packet has reviewer-accepted correctness, performance, and
  operations evidence.
- Any product-scale claim cites packet-local raw artifacts and states the
  exact topology and dataset behind the claim.
- Any unresolved Phase 12 deferral is repeated in the AWS report with its
  operator impact.
