# Feedback: 68 — Visible Active-Candidate Scan Bridge

**Reviewer:** Claude Opus  
**Date:** 2026-04-05

## Overall assessment

The bridge is correct for its stated narrow scope. The active-candidate match in `next_linear_scan_heap_tid` (lines 786–796) correctly uses the candidate's pre-computed score instead of re-scoring when the linear cursor reaches the same element. The ordering of `set_current_scan_result` → `clear_active_scan_candidate` → `store_pending_scan_heaptids` is correct: result state is set, the active slot is freed, and heap tids are loaded for drain.

## Findings

### 1. Bridge code is now superseded by review 69

Review 69 wires `materialize_active_candidate_result` into `amgettuple` *before* the linear scan call. Since materialization clears `active_candidate.score_valid` (line 665), and `materialize_active_candidate_result` also marks the element as emitted (via `set_current_scan_result` → `mark_emitted_element`), the linear scan will skip that element at line 783 (`emitted_contains_element`).

This means the active-candidate match code at lines 786–796 is now dead code — the condition `opaque.active_candidate.score_valid && opaque.active_candidate.element_tid == element_tid` can never be true when reached from the current `amgettuple` flow, because:
- If materialization succeeded: active is cleared, element is emitted, linear skips it
- If materialization failed (element deleted): active is cleared (line 665), condition is false
- If materialization was blocked by pending heaptids: linear drains pending first (line 710–712), never reaches the page loop on the same call

**Recommendation:** Clean up by removing the active-candidate branch from `next_linear_scan_heap_tid` and keeping only the else branch. This avoids confusion about two parallel materialization paths. Not urgent — no correctness impact.

### 2. Score reuse is correct

When the bridge was live (pre-review-69), using `opaque.active_candidate.score` instead of re-calling `score_scan_element_result` was the right optimization — the candidate score was computed during frontier seeding from the same prepared query and element data. No precision concern since the same `score_ip_from_parts` path is used in both cases.

## Verdict

Sound as staged groundwork. The one cleanup opportunity is the now-dead bridge code in the linear scan path.
