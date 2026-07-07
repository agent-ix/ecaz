---
id: SR-002
title: Failure-Domain Analysis of the ec_distann Spec Batch
type: SpecReview
analysis: failure-domain
scope: "spec/functional/index/distann/, spec/non-functional/NFR-017..020, spec/adr/ADR-085, StR-008; re-run against revision d25ea9e0c (co-placed heap rerank, lean record — FR-076/078/079, ADR-085 D11, NFR-018); reconciled at b19551e21 (every finding dispositioned, both former highs downgraded and addressed)"
review_set: all
---
# SR-002: Failure-Domain Analysis — ec_distann Spec Batch

## Summary

As of `b19551e21`, every failure-domain finding in this review has a clear
disposition: all sixteen are now either RESOLVED (the spec fix closes the gap
outright) or ADDRESSED (the spec now specifies the behavior and drills it). The
two findings originally rated **high** for identity/consistency —
FND-001 (epoch-mismatch retry vs in-scan consistency) and FND-002 (insert-time
vec_id collision) — are both **downgraded to low and addressed**: FR-082 now
specifies full-scan restart under the refreshed epoch (with NFR-019 resetting
the BW×H accounting per attempt), and FR-083 defines insert-time identity
collision as an error. The three remaining originally-high findings from the
`d25ea9e0c` two-object rerank split (FND-012 heap-row-absent outcome, FND-013
runtime co-placement drift, FND-016 heap tier under epoch immutability) are all
**RESOLVED**: FR-079 gained the fourth structural-fault outcome and its drills,
FR-078 requires a runtime co-placement check, and FR-082 brings the heap tier
under D10 immutability so `heap_tid` resolves an epoch-frozen vector. The
identity/DML/epoch/beam findings on the record layer (FND-003..FND-011) are
likewise dispositioned by the FR-079/FR-081/FR-082/FR-083 fixes. No finding
remains open.

## Findings

| ID | Severity | Summary | Refs |
|----|----------|---------|------|
| FND-001 | low | ADDRESSED — FR-082 specifies full-scan restart under the refreshed epoch (discard partial state, at most once, then error), FR-082-AC-2 drills it, and NFR-019 resets the BW×H accounting per attempt. Lowered from high. | FR-082, FR-081, NFR-019, NFR-020, EC-019, EC-022 |
| FND-002 | low | ADDRESSED — FR-083 "Insert-time identity collision" errors when the computed vec_id exists with a different source_identity; EC-020 covers hash collision at build AND incremental insert (TC-038/TC-043). Lowered from high. | FR-083, FR-076, ADR-085 D6, EC-020 |
| FND-003 | medium | ADDRESSED — FR-083 UPDATE = tombstone-of-old + insert-of-new under the same vec_id; in-flight scans observe either version, never both (FR-082 visibility rule). | FR-076, FR-083, ADR-085 D6 |
| FND-004 | medium | RESOLVED (b19551e21) — FR-083 delete routes the tombstone write to the hash-owning node and RETAINS the co-placed heap row under FR-082 epoch immutability (reclaimed only at the next epoch build). | FR-083, FR-078, FR-078-AC-4, NFR-020 |
| FND-005 | medium | RESOLVED — FR-083 incremental insert's back-edge re-prune SHALL NOT drop an edge that disconnects a node from the medoid (reachability/FR-077-CON-3 preserved); within-epoch immutability keeps tombstoned records' adjacency intact; next epoch build repairs. | FR-083, FR-077-CON-3, EC-023, NFR-017 |
| FND-006 | medium | RESOLVED — FR-079 now bounds code_threshold as a recall-risk optimization OUTSIDE the correctness guarantees (FR-081-AC-4 early-exit equivalence holds only at NULL), never used where correctness/gate is asserted; default NULL. | FR-079, FR-081-AC-4, NFR-020 |
| FND-007 | medium | ADDRESSED — FR-081 states a beam that exhausts before k returns the fewer-than-k results as a complete result (not a fault); empty index → zero rows. | FR-081, FR-075-AC-3, NFR-020 |
| FND-008 | medium | RESOLVED — FR-082's published-epoch mutation model now enumerates the permitted in-place mutations (tombstone-flag sets, delta-buffer appends, incremental-insert record appends + back-edge amendments) and states the fingerprint attests to roster/placement/format/build-time record set/vector tier — not the mutable delta/tombstone state; the concurrent-mutation visibility rule (FR-082-AC-4) pins reader consistency, and the heap-tier dimension is closed by FND-016. | FR-082, FR-083, ADR-085 D5, ADR-085 D10 |
| FND-009 | low | RESOLVED — FR-082-AC-6: a wedged in-flight count never auto-reclaims; storage retained until the logged operator override. | FR-082, NFR-020 |
| FND-010 | low | ADDRESSED — FR-080 states scans SHALL error if the persisted sample is missing/undecodable (strict policy, no silent medoid fallback). | FR-080, NFR-020 |
| FND-011 | low | RESOLVED — FR-083 bounds per-insert work by the FR-081 traversal cap plus ≤ graph_degree back-edge amendments (the NFR-019 insert-path counterpart). | NFR-019, FR-083 |
| FND-012 | high | RESOLVED — FR-079 adds a fourth outcome (record present, co-placed vector missing → distinct structural fault, case d); EC-024; missing_heap_row drill in TC-042. | FR-079, FR-079-AC-3, FR-076, FR-082, NFR-020 |
| FND-013 | high | RESOLVED — FR-078 requires a runtime co-placement check: heap_tid not resolving node-locally → the FR-079 case (d) structural fault, never a silent skip; EC-025. | FR-078, FR-078-AC-4, FR-079, NFR-020 |
| FND-014 | medium | RESOLVED — FR-079 lets a tombstone return exact_dist NULL with no heap read (results exclude it anyway); FR-083 also retains the heap row under immutability; EC-026. | FR-079, FR-076, FR-076-AC-4, FR-083 |
| FND-015 | medium | RESOLVED — FR-079 maps a failed record or heap read to the corresponding structural fault, non-retriable (distinct from the retriable epoch-mismatch). | FR-079, NFR-020, FR-081 |
| FND-016 | high | RESOLVED — FR-082 brings the heap tier under D10 immutability; heap_tid resolves the epoch-frozen vector, not a base-table TID subject to VACUUM/TID-reuse; +AC-5; EC-027. | FR-082, ADR-085 D10, FR-079, FR-078, NFR-020 |
