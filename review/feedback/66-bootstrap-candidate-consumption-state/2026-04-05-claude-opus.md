# Feedback: Bootstrap Candidate Consumption State

Request:
- `review/66-bootstrap-candidate-consumption-state.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is `maybe_consume_bootstrap_frontier_candidate` the right execution boundary?

**Yes.** The function (scan.rs:664-681) gates on four conditions before consuming:
1. `active_candidate.score_valid` — don't consume if we already have an active candidate
2. `scan_exhausted` — don't consume after scan is done
3. `pending_heaptid_count != 0` — don't consume while draining heap TIDs
4. `scan_dimensions == 0` — don't consume on empty index

These guards are correct. Condition 1 ensures at most one active candidate at a time. Condition 3 prevents consuming a new frontier candidate while the current result is still draining duplicates. This is the right boundary — consumption should only happen when the scan is ready for a new candidate to process.

The consumption itself delegates to `consume_and_refill_bootstrap_frontier` (scan.rs:616-625), which removes the head, refills, and returns the consumed candidate. The consumed candidate is stored in `opaque.active_candidate` (scan.rs:679).

### Interaction with active-candidate state, frontier refill, and heap-tid draining

The interaction is clean:
- Frontier consumption → active candidate (scan.rs:679)
- Active candidate → result materialization (via `materialize_active_candidate_result`, scan.rs:699-710)
- Result materialization → pending heap TIDs (via `store_pending_scan_heaptids`, scan.rs:640)
- Pending heap TID drain → next `amgettuple` result

The active candidate acts as a one-slot buffer between frontier consumption and result production. This prevents the frontier from advancing faster than result production.

### Reset/cleanup correctness

Both exhaustion points (scan.rs:757-758, 847-848) clear `candidate_state`, `active_candidate`, and `result_state` in order. `reset_scan_position` (scan.rs:248-253) clears all three plus expanded/visited/emitted state. No cleanup gap.

## Additional Findings

One observation: the linear scan path (scan.rs:823-827) checks whether the active candidate matches the current element TID during page iteration. If they match, the pre-computed score from frontier consumption is used instead of re-scoring. This is a nice optimization that avoids redundant scoring for elements the frontier already evaluated. The `clear_active_scan_candidate` call at scan.rs:827 ensures the active candidate doesn't match subsequent elements.
