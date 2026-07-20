---
id: NFR-020
title: Distann Fault Behavior
type: NFR
status: PROPOSED
quality_attribute: reliability
relationships:
  - target: "ix://agent-ix/ecaz/FR-078"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-083"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-020: Distann Fault Behavior

## Statement

Under any single read or write fault, an ec_distann operation SHALL return one
complete correct outcome or a classified error.

The ec_distann cluster SHALL NOT expose a partially handed-off or partially
published epoch as active.

## Scope

- Applies to: physical handoff (FR-078), multinode read and materialization
  (FR-079/FR-081), epoch publication/recovery (FR-082), and distributed DML
  (FR-083).
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
- Handoff fault taxonomy: connection loss before and after batch commit; remote
  timeout; backend termination; participant restart; malformed entry; unknown
  wire version; wrong owner; duplicate/out-of-order vec_id; skipped batch
  sequence; identical replay; conflicting replay; schema mismatch; batch digest
  mismatch; final count/digest mismatch; record written without a row-tier
  tuple; row-tier tuple written without a record; and seal with an incomplete
  owner stream.
- Publication fault taxonomy: coordinator crash before the durable publish
  decision; crash after the decision but before any participant publication;
  crash after a strict subset of participants publishes; participant outage
  after the decision; crash after all acknowledgements but before active-pointer
  swap; crash immediately after active-pointer swap; abort racing with recovery;
  retire racing with a coordinator-registered old-epoch scan; and cleanup targeting an active or
  decision-referenced generation.
- Scan-retention fault taxonomy: identical and conflicting coordinator-local
  token replay; epoch-mismatch restart; statement cancellation; backend crash;
  participant restart while a retained generation is readable; normal retire
  racing with a local live scan; coordinator crash after a durable retire
  decision but a strict subset of participant reclaims; and active/non-active
  force-retire racing with a live scan.

## Rationale

A hop-round architecture creates a partial-result hazard: round k of H
failing after k−1 rounds succeeded. Completing "with what we have" would
silently degrade recall — the exact class of silent wrongness (duplicate
top-k, inflated recall) that cost this project weeks on the predecessor
surface. Errors are recoverable; silently wrong results are not.

A streamed physical handoff creates the analogous storage hazard: earlier
batches may be durable when a later batch or participant fails. Hiding Building
and Ready generations, making batches idempotent, and publishing the coordinator
pointer last are required so a recoverable partial build never becomes an
active partial graph.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| fault drill matrix (all cases × scan/insert) | 100% pass | 100% pass | multinode fixture drills |
| epoch-mismatch retry behavior | exactly one refresh-retry then error | same | fault drill assertion |
| wrong-result occurrences under fault injection | 0 | 0 | drill result comparison vs fault-free run |
| handoff fault matrix at begin/batch/seal boundaries | 100% classified error or digest-identical resume | 100% | TC-040/TC-042 fault drills |
| publication crash-boundary matrix | old epoch remains active or new epoch becomes fully acknowledged; no third state | 100% | TC-042 restart drills |
| active epochs with missing/duplicate/non-owned records or row-tier tuples | 0 | 0 | topology audit after every publication drill |
| conflicting retry mutations | 0 additional records/bytes | 0 | receipt/count/byte comparison |
| leaked Building/Ready generations after explicit abort | 0 | 0 | generation inventory after bounded cleanup |
| acknowledged-batch loss after PostgreSQL restart | 0 | 0 | WAL/restart resume drill |
| leaked coordinator scan registrations after normal/error/cancel/restart completion | 0 | 0 | local registry inventory after every TC-042 drill |
| participant pin/unpin query-path operations | 0 | 0 | endpoint/counter assertion |
| duplicate register/release retention-count drift | 0 | 0 | idempotency and conflict drill |
| partial participant reclaim after durable retire decision | recoverable to all reclaimed | 100% | retire-decision restart drill |

## Verification

The distann multinode fixture injects each read, handoff, publication, recovery,
retirement, and DML fault at the named boundary. Each drill compares the active
epoch, participant generation inventory, receipts, record/row counts, digests,
topology audit, and query result against the fault-free state. Drill logs and
normalized result rows land in the owning review packet.

## Acceptance Criteria

### NFR-020-AC-1

A scan fault produces a complete baseline-identical result or a classified
error; no partial result is presented as complete.

### NFR-020-AC-2

A fault before the durable publish decision leaves the prior epoch active and
the new generation resumable or abortable.

### NFR-020-AC-3

A fault after the durable publish decision leaves the prior epoch active until
recovery finishes every participant, then activates the new epoch exactly once.

### NFR-020-AC-4

Retrying an acknowledged handoff or publication operation with identical bytes
does not change counts, digests, or physical bytes.

### NFR-020-AC-5

Conflicting retries, schema drift, wrong ownership, missing coverage, and digest
disagreement fail before publication and never alter the active pointer.

### NFR-020-AC-6

Degraded completion, if introduced later, requires a follow-up FR with explicit
opt-in and result labeling; the default path never degrades silently.

### NFR-020-AC-7

Every non-crash scan exit releases its coordinator-local registration exactly
once, backend death releases registrations with the dead scan, and no scan
performs participant pin/WAL work. Participants reclaim only from a durable
zero-in-flight retire decision; partial application recovers idempotently, and
forced retirement remains an explicit audited non-active-epoch override.

## Dependencies

- **Upstream**: [FR-078](../functional/index/distann/FR-078-distann-hash-placement.md),
  [FR-079](../functional/index/distann/FR-079-distann-remote-expansion-protocol.md),
  [FR-081](../functional/index/distann/FR-081-distann-query-orchestration.md),
  [FR-082](../functional/index/distann/FR-082-distann-epoch-lifecycle.md), and
  [FR-083](../functional/index/distann/FR-083-distann-dml-path.md)
