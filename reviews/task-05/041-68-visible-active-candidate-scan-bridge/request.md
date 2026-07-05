# Request: Visible Active-Candidate Scan Bridge

Commit: `9fa6d6d`

Summary:
- Makes the active-candidate-to-result bridge visible in `amgettuple` without changing the broad linear scan order.
- When the bootstrap linear cursor reaches the same element as the current active candidate, scan execution now materializes that candidate into `current_result` plus pending heap-TID drain state using the candidate-carried score.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- This is the first visible execution step where traversal-groundwork state influences tuple production instead of staying helper-only.
- It keeps the change narrow: no frontier-driven global reordering yet, but the active candidate path is now exercised under real `amgettuple` execution.

Review focus:
- Correctness of the active-candidate / linear-cursor match boundary
- Whether `current_result`, pending heap-TID drain, and active-candidate clearing happen in the right order
- Whether the updated regression still captures the intended narrow staging semantics before broader candidate-driven execution
