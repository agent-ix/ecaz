# Request: Emit Order-By Score From Current Result

Commit: `0a6cdfd`

Summary:
- `amgettuple` now fills `xs_orderbyvals[0]` and `xs_orderbynulls[0]` from the scan-owned `current_result.score` whenever it returns a tuple.
- Adds a debug helper that reads the emitted order-by score directly from the scan descriptor after tuple production.
- Adds pg regression coverage that the emitted order-by value matches the SQL-facing `<#>` score for the returned row.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/am/mod.rs`
- `src/lib.rs`

Why this matters:
- The scan path already tracked the operator-facing score in `current_result`, but tuple production still left the AM order-by output slots empty.
- This slice makes the scan descriptor publish the same score it is already using internally, which is necessary plumbing before planner-visible ordered scans become credible.
- It keeps the current bootstrap-plus-linear execution semantics unchanged while tightening one visible contract of tuple production.

Review focus:
- Whether order-by output emission now stays aligned with `current_result.score` across both bootstrap-candidate and linear-fallback tuple production
- Whether the current allocation/lifetime shape for `xs_orderbyvals` and `xs_orderbynulls` is appropriate for scan descriptor reuse
- Whether any remaining tuple-production paths can still return a tuple without publishing the matching order-by score
