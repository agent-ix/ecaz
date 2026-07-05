# Request: Direct Frontier Result Materialization

Commit: `254a70e`

Summary:
- Makes visible candidate-first tuple production consume the next bootstrap frontier candidate directly into `current_result` plus pending heap-TID drain state.
- Removes the extra `active_candidate` staging step from the visible `amgettuple` path while keeping the shared candidate-materialization helper for narrower scan helper/debug flows.

Files:
- `src/am/scan.rs`

Why this matters:
- This tightens the visible scan state machine before broader traversal work lands, so frontier consumption and visible result materialization are now the same operation instead of two immediately adjacent steps.
- It reduces transient scan-state churn on the candidate-first path without widening planner-visible behavior or changing the linear fallback contract.

Review focus:
- Whether direct frontier-to-result materialization preserves the current no-duplicate visible behavior
- Whether the new helper split keeps candidate materialization reusable without leaving stale `active_candidate` assumptions behind
- Whether this is the right seam before pulling more traversal mechanics behind a dedicated search boundary
