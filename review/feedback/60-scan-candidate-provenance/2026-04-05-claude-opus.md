# Feedback: Scan Candidate Provenance

Request:
- `review/60-scan-candidate-provenance.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Note: Line Numbers Updated

`ScanCandidate` struct is now at scan.rs:1028. `source_tid` field at scan.rs:1030. Semantics unchanged.

## Response to Review Focus

### Does `source_tid` capture the right provenance contract?

**Yes.** Adding `source_tid: page::ItemPointer` to `ScanCandidate` (scan.rs:1030) is the minimal provenance needed for traversal debugging and expansion tracking. It answers "who discovered this candidate?" which is the essential graph-walk provenance.

The entry candidate uses `INVALID` source (scan.rs:496) because it's seeded from metadata, not discovered from another element. Adjacency-discovered candidates record the element TID they were expanded from (scan.rs:690+). This distinction is clear and consistent — `INVALID` means "root of the search", any other TID means "discovered via this element's neighbors."

For multi-hop expansion, this one-hop provenance is sufficient. Full path reconstruction (entry → A → B → C) isn't needed because the expansion logic only cares about the immediate source for visited-set and expanded-source tracking. If path visualization is ever needed for debugging, the source chain could be reconstructed from the sequence of consume operations.

### Debug/provenance assertions

The provenance debug helpers (`debug_candidate_frontier_provenance_slots`, scan.rs:1070+) expose `(element_tid, source_tid)` pairs, which is the right minimal shape for regression assertions. The tests verify that seeded successors record the entry as their source and that refill-discovered candidates record the consumed head as their source. These are the important parent-tracking edges without overfitting to a specific graph topology.

## Additional Findings

No issues found. Clean structural addition.
