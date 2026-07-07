---
id: SR-006
title: "risk-complexity analysis of the ec_distann spec batch"
type: SpecReview
analysis: risk-complexity
scope: "spec/stakeholder/StR-008, spec/functional/index/distann/FR-075..FR-083, spec/non-functional/NFR-017..NFR-020, spec/adr/ADR-085"
review_set: all
---

## Summary

Technical-risk and volatility scoring of the 15-artifact ec_distann batch
before tasking. The lane's risk profile is unusually legible because ADR-085
already names its own hazards (D1–D9) and the milestone order (M0 storage/
sensitivity → M2 multinode measurement → M5 incremental insert) puts the two
highest-risk novel pieces — D1 space arithmetic at M0 and the sharded-build
stitch ahead of the multinode gate — appropriately early, with the hardest
distributed-consistency work (FR-083 incremental insert) correctly gated
last where its failure cannot invalidate the read-path program.

**Risk register** (both axes scored for every artifact; High entries carry a
named mitigation):

| Req | Tech Risk | Volatility | Drivers | Mitigation |
|-----|-----------|------------|---------|------------|
| StR-008 | High | Low | Program bet: successor to a measured-dead architecture; satisfaction = NFR-017 gate | Pre-registered gate protocol reusing the Task 146 anchor run; kill criteria explicit (see FND-002) |
| FR-075 | Low | Low | Pattern-follows ec_diskann handler/opclass surface | — |
| FR-076 | Medium | High | Format is expected to churn: D1 fallback is a version bump, D7 codec is an open measured choice | Research rebuild posture (no migration); M0 freezes D1/D7 before dependents harden (see FND-006) |
| FR-077 | High | Medium | **Least-proven step in the program** (ADR-085 Consequences): stitch must reproduce monolithic graph quality within 0.001 recall; closure_epsilon under discovery | Property tests (CON-1..4: degree cap, uniqueness, reachability), A/B vs monolithic build, D8 streaming bound; monolithic fallback exists but is unstated (FND-001) |
| FR-078 | Low | Low | Deterministic hash, no per-record directory; roster change = rebuild (acceptable for research) | — |
| FR-079 | Medium | Medium | Reuses lifted SPIRE transport/pool; new error taxonomy (placement vs tombstone vs missing); response shape is coupled to D1 (FND-006) | Fault-drill ACs; epoch fingerprint validation |
| FR-080 | Medium | Medium | In-memory head index reuses SPIRE top-graph builder; C default deliberately unfrozen until M0 (D3) | D3 sensitivity measurement gates the default; M0 single-shard degenerate case flagged in SR-001 FND-003 |
| FR-081 | High | Low | Distributed coordination core: parallel per-node hop rounds, early-exit correctness (AC-4), hard BW×H cap | Reuses ADR-056 eager-scan + post-142 pooling (proven); 2-node vs 1-node identity test (AC-1); early-exit A/B |
| FR-082 | Medium | Low | Epoch lifecycle is lifted SPIRE machinery (proven in production-parity drills); in-scan consistency + reclaim gating are known patterns | Lifecycle/fault drills (AC-1..3) |
| FR-083 | High | Medium | Distributed multi-writer read-modify-write back-edges, mid-insert fault atomicity, insert/query concurrency — the hardest consistency problem in the batch | Correctly last (M5); interim D5 posture ships first; fault + concurrency drills (AC-5/6); recall-parity A/B vs rebuild (AC-4) |
| NFR-017 | High | Low | Hard performance guarantee and the program kill criterion: p50 ≤ IVF anchor at matched recall, distinct_recall ≥ 0.999 at three scales; floor is H × per-round RTT | D4 reopen trigger (baton passing) if RTT ≥ 50% of p50; needs an earlier kill-check spike (FND-002) and a matched-recall definition (FND-005) |
| NFR-018 | High | Low | D1 arithmetic at stated defaults appears to already breach the 4× threshold (FND-003) | M0 storage measurement is the first gate; D1 fallback format is a named, cheap hedge |
| NFR-019 | Medium | Low | Bound is satisfiable by construction (hard cap); the real hazard is budget-needed-for-recall growing with scale (FND-004) | Cross-scale ratio row exists; strengthen per FND-004 |
| NFR-020 | Medium | Low | Broad fault matrix, but reuses the multinode drill machinery; new cases (hop_round_failure_mid_beam, mid-insert) are additive | Error-or-complete posture (no silent degradation); drills per case |
| ADR-085 | — | Medium | D2 loopback-only gate substrate limits external validity; D3/D7 defaults intentionally unfrozen until M0 | netem informational run (D2); M0 measurements freeze D3/D7 |

**Top hazards** (review live before plan generation): FND-001 (stitch),
FND-002 (hop-round latency floor / kill criterion timing), FND-003 (D1
space arithmetic), FND-004 (NFR-019 tautology risk), FND-005 (matched-recall
ambiguity in the gate).

**Kill criteria**: NFR-017 is the only true program-invalidating requirement
— it is StR-008's satisfaction bar and there is no fallback architecture
left (partitioned SPIRE is shelved-with-evidence; IVF-only distributed is
recorded as viable but does not meet the need at high recall). FR-077
failure degrades to a monolithic build (slower build, program survives at
research scales); NFR-018 breach flips D1's fallback format; FR-083 failure
strands the final milestone but leaves the read-path result intact. Hedging
is generally good: D1 names a concrete fallback format, D4 names a
quantified reopen trigger, D5 stages the write path, D6 fails builds on
collision rather than assuming, and the research rebuild posture makes
format churn cheap.

**Failure-domain cross-check**: no `spec-failure-domain-analysis`
deliverable exists for this batch yet; NFR-020's fault taxonomy plus FR-079's
placement/tombstone/missing distinction covers the topology and identity
domains informally. Open gap: none blocking, but the mid-insert atomicity
domain (FR-083-AC-5) deserves a dedicated pass before M5 tasking.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | high | FR-077 sharded-build stitch is the least-proven step and the batch's highest implementation risk (novel algorithm, 0.001 recall-parity bar vs monolithic, closure_epsilon under discovery). Mitigations are strong (CON-1..4 property tests, A/B, D8 streaming) but the spec never states the obvious risk bound: at research scales a monolithic single-shard build is a working fallback that preserves every downstream FR. State it so a stitch slip degrades build parallelism, not the program | FR-077, ADR-085 |
| FND-002 | high | NFR-017 is the program kill criterion (p50 ≤ 37.6 ms IVF anchor at matched recall) yet its dominant risk — the H×RTT latency floor — is first measurable at M2, after the format, build, and transport investment. Add an M0/M1 kill-check spike: the recall-vs-H curve is measurable on a single-node build, and per-round transport cost is already known from the post-142 SPIRE pooling work; their product bounds multinode p50 before any multinode code exists. D4's reopen trigger (baton passing at RTT ≥ 50% of p50) is a good hedge but fires late | NFR-017, FR-081, ADR-085 |
| FND-003 | high | ADR-085 D1's own arithmetic likely breaches NFR-018 at the stated defaults: at dim=1536, R=32, 4-bit codes the neighbor-code block alone is ~24.6 KB against 6.1 KB raw vector — ~5× with the vector and adjacency included, over the ≤4.0 threshold and well over the ≤3.0 target. The hedge (D1 fallback: adjacency-only records, codes piggybacked on responses) is named and cheap under the rebuild posture, but the specs should acknowledge that defaults probably need R lower, codes smaller, or the fallback — M0's storage cell is correctly the first measurement | NFR-018, FR-076, ADR-085 |
| FND-004 | medium | NFR-019's headline metric (expanded ≤ BW×H at fixed GUCs; cross-scale ratio ≤ 1.1 at fixed BW,H) is satisfied by construction — the loop hard-caps at BW×H, so the ratio row cannot detect the SPIRE failure mode recurring as budget-needed-for-recall growing with corpus size. NFR-017's 0.999-at-three-scales requirement catches it only if the sweep is held fixed across scales; add an explicit "minimum BW×H achieving 0.999 distinct recall, per scale" row to the gate manifest so the corpus-independence claim is measured, not assumed | NFR-019, NFR-017, StR-008 |
| FND-005 | medium | NFR-017's "at matched recall" is ambiguous: the IVF anchor is 0.9980 distinct recall @ 37.6 ms, but ec_distann must hold ≥ 0.999 — a recall point where IVF has no measured latency. Define the comparison rule (e.g. compare at the anchor's 0.9980 operating point, or extend the IVF sweep to 0.999 in the four-way table) before the gate run, else the gate verdict is contestable | NFR-017, StR-008 |
| FND-006 | medium | FR-076 is deliberately high-volatility (D1 fallback is a format-version flip; D7 codec is an open measured choice), but the D1 fallback also changes FR-079's response semantics (neighbor codes move from embedded-in-record to piggybacked-on-response) — a cross-FR coupling the specs don't surface. Keep FR-079's wire contract stated independently of record layout so a D1 flip after M0 is a storage change, not a protocol change | FR-076, FR-079, ADR-085 |
| FND-007 | medium | FR-083 incremental distributed insert (batched remote read-modify-write back-edges, mid-insert atomicity, insert/query concurrency) is the highest-consistency-risk FR and is correctly gated last (M5), where failure cannot invalidate the read-path program. But because it is committed scope ("in scope, not conditional"), the spec should state whether the D5 interim posture is an acceptable terminal state if M5 slips — otherwise the program's definition of done is hostage to its riskiest tail milestone | FR-083, ADR-085 |
| FND-008 | low | D2 gates on loopback multi-instance only, with injected-latency (netem) informational — internally consistent (the IVF/HNSW anchors were measured the same way) but any generalization of the gate verdict to real networks rides entirely on the ungated H×RTT sensitivity report; keep that report mandatory in the gate packet | NFR-017, ADR-085 |
| FND-009 | low | Low-risk, low-volatility set confirmed: FR-075 (pattern-follows ec_diskann), FR-078 (deterministic hash, directory-free), FR-082 (lifted, drill-proven SPIRE epoch machinery). These are safe early tasking targets and need no special hedging | FR-075, FR-078, FR-082 |
| FND-010 | low | FR-080's D3 (head_index_cap default frozen only after M0 sensitivity measurement) is deliberate, well-hedged volatility; note the already-filed SR-001 FND-003 ordering wrinkle (M0 head index precedes sharded build) when sequencing tasks | FR-080, ADR-085 |
