# Request: Explicit Bootstrap Completion At Phase Dispatch

Commit: `a576cb0`

Summary:
- move bootstrap completion out of the bootstrap selector helper in `src/am/scan.rs`
- make `try_select_next_bootstrap_frontier_result` only attempt selection
- let the phase dispatcher decide explicitly when bootstrap is complete and when same-call linear fallback may begin

Please review:
- whether moving bootstrap completion up into the phase dispatcher makes the staged executor easier to reason about without changing behavior
- whether same-call fallback from exhausted bootstrap into linear selection is still preserved exactly where intended
- whether the remaining test-only bootstrap materialization wrapper is still the right place to preserve old debug semantics
