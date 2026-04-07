# Request: Gate Frontier-Head Candidate Accessor

Commit: `f6b09f9`

Summary:
- make the full `current_candidate_frontier_head` accessor in `src/am/scan.rs` available only to `test` / `pg_test` builds
- keep production runtime selection on a private internal `candidate_frontier_head` helper
- leave the existing debug/test behavior unchanged

Please review:
- whether any non-debug production path still needs the full candidate-returning frontier-head accessor
- whether the private/public split now makes the runtime vs debug boundary clearer
- whether this is the right next step in shrinking scan’s production-visible bootstrap helper surface
