---
id: NFR-020
title: Distann Fault Behavior
type: NFR
status: PROPOSED
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-020: Distann Fault Behavior

## Statement

Under any single fault (connection reset, remote timeout, remote backend
termination, epoch mismatch, missing record, placement drift, network
partition, mid-insert failure), an ec_distann scan SHALL either return a
correct complete result or raise an error — it SHALL NOT return a partial or
stale result presented as complete.

## Scope

- Applies to: the multinode read path (FR-079/FR-081), epoch lifecycle
  (FR-082), and the incremental insert path (FR-083).
- Fault taxonomy: the reused multinode drill cases
  (connection_reset_mid_batch, epoch_mismatch, remote_statement_timeout,
  remote_backend_termination, missing_or_reindexed_remote_index, and
  simulated network partition via the existing fixture's
  `simulated_network_partition` mechanism — connection-level injection, as
  true interface partition is not injectable on the loopback fixture) plus
  the distann-specific cases hop_round_failure_mid_beam,
  missing_node_record, placement_drift, mid-insert failure, and mid-delete
  failure (a lost remote tombstone write must error, never silently
  resurrect the row).

## Rationale

A hop-round architecture creates a new partial-result hazard: round k of H
failing after k−1 rounds succeeded. Completing "with what we have" would
silently degrade recall — the exact class of silent wrongness (duplicate
top-k, inflated recall) that cost this project weeks on the predecessor
surface. Errors are recoverable; silently wrong results are not.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| fault drill matrix (all cases × scan/insert) | 100% pass | 100% pass | multinode fixture drills |
| epoch-mismatch retry behavior | exactly one refresh-retry then error | same | fault drill assertion |
| wrong-result occurrences under fault injection | 0 | 0 | drill result comparison vs fault-free run |

## Verification

The distann multinode fixture implements every fault case as an automated
drill; each drill asserts the scan/insert either errors or returns results
identical to the fault-free baseline. Drill logs land in the owning review
packet.

## Acceptance Criteria

Degraded-completion modes, if ever introduced, SHALL be opt-in, labeled in
the result metadata, and specified by a follow-up FR — the default path
never degrades silently.

## Dependencies

- **Upstream**: [FR-079](../functional/index/distann/FR-079-distann-remote-expansion-protocol.md),
  [FR-081](../functional/index/distann/FR-081-distann-query-orchestration.md),
  [FR-082](../functional/index/distann/FR-082-distann-epoch-lifecycle.md)
