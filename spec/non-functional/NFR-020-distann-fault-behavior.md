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
- Read/DML fault taxonomy — the shipped fixture drill matrix (12 drills, as
  implemented in `distann_multicluster.rs`):
  `simulated_network_partition` (connection-level injection, as true
  interface partition is not injectable on the loopback fixture),
  `epoch_bump_no_false_reject`, `remote_content_divergence`,
  `missing_or_reindexed_remote_index`, `remote_backend_termination`,
  `placement_drift`, `remote_statement_timeout`,
  `hop_round_failure_mid_beam`, `missing_node_record`,
  `missing_heap_row_co_placement_drift`,
  `mid_delete_lost_tombstone_no_resurrect` (a lost remote tombstone write
  must error, never silently resurrect the row), and
  `mid_insert_failure_rolls_back`. Two names from a prior revision are
  rebased: `connection_reset_mid_batch` has no drill (open obligation), and
  `epoch_mismatch` is covered by the pair `remote_content_divergence`
  (content-fingerprint mismatch fails closed) plus
  `epoch_bump_no_false_reject` (a bare epoch-number bump does not falsely
  reject).
- The three boundary taxonomies below are the **specified** fault surface.
  Only a subset has an injecting drill today; the Verification section splits
  the taxonomies into drilled-today versus open obligations, and the 100%
  metrics below apply to the drilled subset only.
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
| fault drill matrix (shipped 12-drill matrix × scan/insert) | 100% pass | 100% pass | multinode fixture drills |
| epoch-mismatch retry behavior | exactly one refresh-retry then error | same | fault drill assertion (retry-count assertion is an open obligation — implemented in `scan.rs`, no drill asserts the count) |
| wrong-result occurrences under fault injection | 0 | 0 | drill result comparison vs fault-free run |
| handoff fault matrix at begin/batch/seal boundaries | 100% classified error or digest-identical resume | 100% | TC-040/TC-042 fault drills |
| publication crash-boundary matrix | old epoch remains active or new epoch becomes fully acknowledged; no third state | 100% | TC-042 restart drills |
| active epochs with missing/duplicate/non-owned records or row-tier tuples | 0 | 0 | topology audit after every publication drill |
| conflicting retry mutations | 0 additional records/bytes | 0 | receipt/count/byte comparison |
| leaked Building/Ready generations after explicit abort | 0 | 0 | generation inventory after bounded cleanup |
| acknowledged-batch loss after PostgreSQL restart | 0 | 0 | WAL/restart resume drill (open — no drill yet) |
| leaked coordinator scan registrations after normal/error/cancel/restart completion | 0 | 0 | local registry inventory after every TC-042 drill |
| participant pin/unpin query-path operations | 0 | 0 | endpoint/counter assertion (open — no such counter exists yet) |
| duplicate register/release retention-count drift | 0 | 0 | idempotency and conflict drill (open — no drill yet) |
| partial participant reclaim after durable retire decision | recoverable to all reclaimed | 100% | retire-decision restart drill |

## Verification

The distann multinode fixture injects each drilled fault at its named boundary.
Each drill compares the active epoch, participant generation inventory,
receipts, record/row counts, digests, topology audit, and query result against
the fault-free state. Drill logs and normalized result rows land in the owning
review packet.

**Drill coverage status (audited 2026-08-01).** The fault-behavior
requirements above are normative for every named boundary; the drill fixture
covers a subset:

- **Drilled today — read/DML**: the 12-drill matrix listed in Scope.
- **Drilled today — build/publication/retirement**: owner-outage partial-build
  (Task 198), participant-down publish and post-ack/pre-pointer publish
  restarts, DROP EXTENSION precondition, and the Task 199 drills (ENOSPC,
  mid-scan fallback, corruption fallback, real-INSERT invalidation,
  retirement/reclaim, mutation fence).
- **Specified boundary, no drill yet (open obligations)**: pre-decision
  coordinator crash; abort racing with recovery; duplicate register/release
  retention-count drift; WAL/restart batch resume; the handoff
  malformed-entry, unknown-wire-version, and identical/conflicting-replay
  cases; the pin/unpin counter assertion (no counter exists); and the
  epoch-mismatch single-retry count assertion (the retry is implemented in
  `scan.rs` but no drill asserts the count).

A packet claiming 100% pass SHALL scope that claim to the drilled subset; the
open-obligation boundaries are unverified, not passed.

## Acceptance Criteria


| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-020-AC-1 | A scan fault produces a complete baseline-identical result or a classified error; no partial result is presented as complete. | Demonstration |
| NFR-020-AC-2 | A fault before the durable publish decision leaves the prior epoch active and the new generation resumable or abortable. | Demonstration |
| NFR-020-AC-3 | A fault after the durable publish decision leaves the prior epoch active until recovery finishes every participant, then activates the new epoch exactly once. | Demonstration |
| NFR-020-AC-4 | Retrying an acknowledged handoff or publication operation with identical bytes does not change counts, digests, or physical bytes. | Demonstration |
| NFR-020-AC-5 | Conflicting retries, schema drift, wrong ownership, missing coverage, and digest disagreement fail before publication and never alter the active pointer. | Inspection |
| NFR-020-AC-6 | Degraded completion, if introduced later, requires a follow-up FR with explicit opt-in and result labeling; the default path never degrades silently. | Demonstration |
| NFR-020-AC-7 | Every non-crash scan exit releases its coordinator-local registration exactly once, backend death releases registrations with the dead scan, and no scan performs participant pin/WAL work. Participants reclaim only from a durable zero-in-flight retire decision; partial application recovers idempotently, and forced retirement remains an explicit audited non-active-epoch override. | Demonstration |

## Dependencies

- **Upstream**: [FR-078](../functional/distann/build/FR-078-distann-hash-placement.md),
  [FR-079](../functional/distann/read/FR-079-distann-remote-expansion-protocol.md),
  [FR-081](../functional/distann/read/FR-081-distann-query-orchestration.md),
  [FR-082](../functional/distann/lifecycle/FR-082-distann-epoch-lifecycle.md), and
  [FR-083](../functional/distann/lifecycle/FR-083-distann-dml-path.md)
