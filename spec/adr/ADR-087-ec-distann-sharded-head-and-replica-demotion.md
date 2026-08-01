---
type: ADR
id: ADR-087
title: "ec_distann: Sharded Membership-Only Head; Traversal Replica Demoted to Non-Conforming Opt-In"
status: ACCEPTED
impact: Supersedes ADR-086's promotion decision and amends ADR-085 decision item 5 — the shipped default becomes the sharded membership-only head with bounded gateway copies; the coordinator traversal replica is demoted to a non-conforming, never-decision-bearing opt-in.
date: 2026-08-01
---
# ADR-087: ec_distann — Sharded Head Default and Traversal-Replica Demotion

## Context

ADR-085 (decision item 5, D3) chose a coordinator-resident head index, and
ADR-086 accepted a coordinator-resident full traversal replica with a
measured PROMOTE decision toward "Ready-replica preference as the normal
path". Both decisions predate two findings:

1. **Task 203 (paper conformance)**: the replica program had abandoned
   distribution — a coordinator holding every graph record and vector is
   O(N) coordinator state, exactly what DISTRIBUTEDANN's architecture
   avoids; and the head had drifted from the paper's per-partition design.
2. **Task 210 (distribution invariant ratchet)**: NFR-021 was tightened to
   a closed allowlist — `NFR_021_KNOWN_DISTRIBUTION_GAPS` is deleted, and
   any coordinator-resident unsharded relation with non-zero derived bytes
   hard-fails the suite. The former coordinator-head exemption (NFR-021
   clause 3) was revoked.

Task 210 then shipped and measured the conforming replacement:

- **Sharded, membership-only head as the default**
  (`ec_distann.shard_head_storage` / `ec_distann.sharded_head_search`
  default on): the coordinator persists one bounded membership blob
  (4 + 8·C bytes) and zero landmark vectors; owners serve their head shards
  from locally held vectors; §4.1-style head-shard replicas with population
  attestation spread serving load. Coordinator head relations audit at
  0 bytes with `outstanding_distribution_gap=none`
  (`reviews/task-210/006-zero-byte-head/`).
- **Bounded gateway copies (TRAV-30)** — the direction ADR-086 had listed
  under rejected alternatives — as the conforming answer to per-hop
  response cost: response bytes −36/−9/−7% at 10/50/100k with identical
  recall (`reviews/task-210/004-gateway-copies/`).
- The sharded head's residual cost vs the (non-conforming) local-head
  referent: +8.6% @10k, −0.5% @50k, +5.1% @100k mean latency — the quantity
  Tasks 212 (crown cache) and 213 (fused head hop) target.

## Decision

1. **The sharded, membership-only head is the shipped default** for
   multi-owner rosters, normatively owned by FR-080 (as rewritten under
   Task 214). ADR-085 decision item 5 and the architecture half of D3 are
   amended accordingly; D3's cap-retention measurement (C = 4096) stands,
   and ADR-085's core single-global-graph decision (items 1–4, D8) is
   unchanged.
2. **ADR-086's promotion decision is superseded.** The coordinator
   traversal replica remains implemented (FR-084) but is demoted to a
   non-conforming opt-in: reachable only via
   `ec_distann.allow_nonconforming_replica` (default off), never selected
   by default, and never a decision-bearing benchmark arm (NFR-021 clause
   4, NFR-022). "Ready-replica preference as the normal path" will not
   ship.
3. **TRAV-30 bounded gateway copies are selected** as the conforming
   replacement direction for coordinator-side traversal acceleration,
   normatively owned by FR-086. The same bounded, codes-only,
   rebuild-only class is the design envelope for the Task 212 crown cache
   and the Task 213 fused head hop.

## Consequences

- Benchmark controls: the legacy coordinator-local head survives only as
  the fixture `--local-head` control arm and is inadmissible as a decision
  control (NFR-022); the replica may ride as context only.
- The head's residual fan-out RTT is an accepted, measured cost at this
  decision point; recovering it is explicitly Tasks 212/213 scope under the
  FR-086 conformance envelope, not a reason to revisit decisions 1–2.
- Every future coordinator-resident structure MUST pre-register under the
  NFR-021 storage-class scheme (`coordinator_resident_unsharded` /
  `bounded` / `control`) and ship activation counters asserted non-zero in
  its A/B.

## Supersedes / Amends

- Supersedes: ADR-086 Decision and its embedded Task-198 promotion record
  (the replica object model, digest chain, and invalidation protocol in
  ADR-086 remain accurate as implementation history for FR-084).
- Amends: ADR-085 decision item 5 and D3 (architecture portion only).
