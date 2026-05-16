# Task 30 Phase 13d: SPIRE Read Efficiency and Observability

Status: implementation checkpoint ready for review
Owner: coder1 / SPIRE AWS verification track
Priority: P1 before first AWS read workload pass

## Goal

Phase 13d closes the final algorithmic and observability gaps found in the
post-13c efficiency review before AWS testing. Measurement is the first
priority: the production CustomScan read path must expose where time is spent
before we use external infrastructure. Low-risk local optimizations that reduce
duplicated remote work are second priority.

## Scope

- [x] Add a live production read profile surface for the same
  `EcSpireDistributedScan` heap-resolution path used by normal distributed
  reads.
- [x] Attribute elapsed time and counts for planning, fingerprint guard,
  conninfo secret lookup, connect/TLS, statement-timeout setup, remote regclass
  lookup, endpoint identity, candidate receive, heap receive, tuple payload
  decode, merge, strict failures, remote timeout/cancel, and degraded skips.
- [x] Reuse each remote libpq session across candidate receive and heap
  receive in the live production read path so each dispatch performs one
  conninfo lookup, one socket/TLS handshake, one statement-timeout setup, one
  remote index lookup, and one endpoint-identity query.
- [x] Keep candidate and heap remote I/O overlapped across nodes so simulated
  AWS latency does not serialize per-remote heap fetches.
- [x] Keep ordinary operator diagnostics cheap by moving full heap-resolution
  execution behind the explicit profile/read-summary surfaces.
- [x] Bound post-dedupe merge work with partial selection before the final
  deterministic sort/truncate.
- [x] Add focused tests for metric rollup and duplicate-count merge behavior.

## AWS Simulation Notes

Local multi-instance testing is enough to validate correctness, counters,
strict/degraded branches, TLS setup, and whether candidate/heap stages still
overlap. It cannot fully predict AWS variance from cross-AZ RTT, ENA queueing,
Secrets Manager latency, CPU steal, EBS stalls, or EC2-to-EC2 TLS handshake
costs. Phase 13d therefore adds per-stage production-read measurements instead
of trying to encode all AWS behavior into local fixtures.

The AWS pass should capture `ec_spire_remote_search_production_read_profile`
beside the existing latency and pipeline artifacts for every read workload row.
If the first AWS run shows a dominant bucket that local latency simulation did
not reproduce, open a follow-up packet with the packet-local profile rows and
the corresponding raw benchmark log.

## Acceptance

- [x] `ec_spire_remote_search_production_read_profile(index_oid, query, top_k)`
  exposes live production read timing and count metrics without leaking
  conninfo secrets.
- [x] The production CustomScan heap-resolution path no longer opens separate
  remote libpq sessions for candidate receive and heap receive.
- [x] The default operator diagnostics path no longer performs full live heap
  resolution as a side effect.
- [x] Merge results stay deterministic while avoiding a full sort of discarded
  rows when the candidate set is larger than `top_k`.
- [ ] Reviewer accepts the 13d packet before AWS read workload execution.
