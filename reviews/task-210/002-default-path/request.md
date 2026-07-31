# Task 210 P1 review request: the sharded owner path is the default, provably

- Branch: `task-203-ec-distann-conformance`
- Code commit under review: `e5047081a` (make the sharded owner path the
  default — `generation_read.rs` opens `ReadyTraversalReplica` only behind
  `ec_distann.allow_nonconforming_replica`, default off; no replica code
  deleted), plus the suite-side rejection of replica arms that claim
  conformance or a decision-bearing role.
- Evidence: this packet's `artifacts/manifest.md` +
  `artifacts/run/results.jsonl` (10k/50k/100k A/B, run 2026-07-31).

## What P1 claims and how the run proves it

NFR-021 clause 4: no read path silently substitutes a non-distributed
structure for a distributed one. Clause 5: it holds in the shipped default
configuration.

The evidence design (handoff §5.2): the candidate arm runs the shipped
default **on a cluster that holds a Ready FR-084 replica image**, built by an
explicit-opt-in context arm on the same cluster minutes earlier. Result:

- context arm (GUC on): `replica_scans = 50/50` — the image is real and
  serves scans when asked;
- candidate arm (GUC off, same image): `replica_scans = 0` at every scale,
  recall bit-identical to a control cluster that never built an image;
- conformance rows: control/candidate `conforming, decision_eligible=true`;
  context `nonconforming, decision_eligible=false` with its 1.66 GB
  coordinator copy itemised.

Latency: candidate within noise of control at 10k/100k; a +13% mean delta at
50k is recorded and discussed in the manifest (recall equality and
`replica_scans=0` rule out a path change; the deviation is opposite in sign
to silent replica use).

## Scope notes

- These arms ran the pre-P2-promote default (coordinator-local head), so the
  known task-210-P2 head-gap allowlist entries appear on every row; closing
  them belongs to the P2 promote decision, not this packet.
- Focused PG18 validation of the P1 gate: the six fixture clusters exercised
  scan open, custom scan, serving and reconciliation checks on PG18 under
  both GUC states; unit-level gate behaviour is covered by the suite's
  rejection tests (`cargo test -p ecaz-cli ... distann_`, 29 pass).

Request open.
