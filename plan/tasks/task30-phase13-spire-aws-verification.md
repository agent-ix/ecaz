# Task 30 Phase 13: SPIRE AWS Verification

Status: blocked on final local readiness
Owner: coder1 / SPIRE AWS verification track
Priority: after Phase 12 exit criteria are met

## Sub-phases

Phase 13 is decomposed into a design phase and a runbook phase. The
entry/exit gate below stays in this file; topology, datasets, workload
matrix, thresholds, observability surface, fault drills, packet
skeleton, and operator surface live in 13a; the executable runbook,
Terraform module, and helper scripts live in 13b.

- [Phase 13a — SPIRE AWS Verification Design](task30-phase13a-spire-aws-verification-design.md)
- [Phase 13b — SPIRE AWS Verification Runbook](task30-phase13b-spire-aws-verification-runbook.md)

## Goal

Run AWS-cloud-class verification only after the local CustomScan
distributed path has passed Phase 12 hardening. Phase 13 is the
external-scale evidence phase, not a place to discover known local
hardening gaps. The "AWS/RDS-class verification" wording inherited from
the original gate is amended in Phase 13a.1.a to "AWS-cloud-class
verification on self-managed EC2 PG18" because RDS and Aurora do not
permit loading the ecaz extension's custom AM / CustomScan.

## Entry Gate

- [x] Phase 12 is complete or every remaining Phase 12 item has an accepted
  reviewer deferral.
- [x] Phase 12c.4 READ schema-drift guard has landed with coord-only,
  remote-only, and both-sides CustomScan fixtures. Evidence: Phase 12c
  packet `763`.
- [ ] The final local production-readiness bundle passes from clean setup.
- [ ] Operator runbook covers typed tuple transport, 2PC recovery,
  `max_prepared_transactions`, strict/degraded behavior, local capacity
  targets, and known v1 limitations.
- [ ] AWS packet manifest is prepared before allocating or running external
  infrastructure.
- [ ] Phase 13a (design) is reviewer-accepted; every box in 13a.1..10
  is decided or recorded as a deferral.
- [ ] Phase 13b.1 deliverables are committed: `infra/spire-aws/`
  (Terraform module + Makefile) and `scripts/spire-aws/` (orchestration
  shell scripts, SQL helpers, suite configurations).
- [ ] Phase 13.0 counter prerequisites (Phase 13a.5.1..4) are landed or
  each deferral carries the operator-impact note that Phase 13b will
  repeat.

## Non-Goals

- Implementing missing Phase 12 hardening after AWS has already started.
- Claiming billion-scale product readiness from a partial local-readiness
  bundle.
- Reopening multi-coordinator HA, cross-shard non-vector query execution, DDL
  propagation, or cross-shard embedding UPDATE moves unless separate ADRs have
  already accepted that scope.

## Verification Plan

The detailed plan now lives in
`task30-phase13a-spire-aws-verification-design.md` (decisions) and
`task30-phase13b-spire-aws-verification-runbook.md` (procedure).
The legacy bullet list below is preserved for traceability against the
original gate phrasing; each item is now owned by the cited Phase 13a
section.

- [ ] Define the AWS-cloud topology (Phase 13a.1):
  - [ ] coordinator instance class and storage;
  - [ ] remote instance classes and storage;
  - [ ] network placement;
  - [ ] PostgreSQL version and extension build;
  - [ ] security boundary and conninfo-secret setup.
- [ ] Define datasets (Phase 13a.2):
  - [ ] small correctness dataset;
  - [ ] representative local-to-AWS scale dataset;
  - [ ] optional larger stress dataset only after smaller gates pass.
- [ ] Define workload matrix (Phase 13a.3):
  - [ ] vector ORDER BY LIMIT reads through CustomScan;
  - [ ] coordinator-routed INSERT;
  - [ ] non-embedding UPDATE;
  - [ ] DELETE;
  - [ ] PK SELECT;
  - [ ] degraded-mode read behavior;
  - [ ] strict-mode fail-closed behavior.
- [ ] Capture correctness evidence (Phase 13a.4):
  - [ ] remote rows returned through `EcSpireDistributedScan`;
  - [ ] placement directory matches remote heap ownership;
  - [ ] no materialization catalog/register calls;
  - [ ] source identity and boundary-replica dedupe remain stable;
  - [ ] Stage E fault/lifecycle smoke subset passes in AWS topology.
- [ ] Capture performance evidence (Phase 13a.4 + 13a.5):
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
- [ ] Capture operations evidence (Phase 13a.6 + 13b.9):
  - [ ] clean setup and teardown transcript;
  - [ ] extension install/upgrade transcript;
  - [ ] runbook recovery drill for a synthetic orphaned prepared transaction,
    unless explicitly deferred;
  - [ ] required GUC verification on every remote;
  - [ ] sanitized error behavior for auth/certificate/conninfo failures.
- [ ] Publish packet-local artifacts for every AWS run (Phase 13a.7 + 13b.10):
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
