# Request: Remove Redundant Frontier-Head Tid Helper

Commit: `3bed8f6`

Summary:
- remove `current_candidate_frontier_head_tid` from `src/am/scan.rs`
- have unit tests and `src/am/scan_debug.rs` derive the head TID directly from the existing debug-only candidate-returning accessor
- shrink one more redundant debug helper from the scan module surface

Please review:
- whether the TID-only helper was truly redundant with the candidate-returning frontier-head accessor
- whether the updated debug/test call sites still express the frontier-head contract clearly
- whether this is the right next step in reducing scan’s debug-only helper duplication
