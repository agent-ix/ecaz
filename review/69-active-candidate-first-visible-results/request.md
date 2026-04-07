# Request: Active Candidate First Visible Results

Commit: `f2ad000`

Summary:
- Lets `amgettuple` materialize an active bootstrap candidate into visible tuple production before falling back to the linear page scan.
- Adds scan-owned emitted-element tracking so the later linear pass skips any element already returned through that candidate-first path.
- Updates scan regressions to assert "every heap tid exactly once" while the visible order intentionally moves away from pure linear page order.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- This is the first scan slice where candidate state can drive visible result order instead of only influencing helper-local state.
- The emitted-element guard is the safety seam that keeps this narrow step from duplicating results when linear scan later reaches the same element.

Review focus:
- Correctness of candidate-first materialization before `next_linear_scan_heap_tid`
- Whether emitted-element tracking is the right de-dup boundary for this stage
- Whether the updated lifecycle and frontier tests still capture the intended narrow semantics before broader candidate-driven traversal
