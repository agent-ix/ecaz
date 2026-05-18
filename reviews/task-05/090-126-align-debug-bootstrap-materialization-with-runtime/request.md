# Request: Align Debug Bootstrap Materialization With Runtime Path

Commit: `e6c2c2d`

Summary:
- The debug/bootstrap materialization helper now reads the current visible frontier head and then drives bootstrap result production through the same `materialize_next_bootstrap_frontier_result` helper that `amgettuple` uses.
- The old `consume_and_refill_bootstrap_frontier` helper is now fenced to test/pg-test use only, because runtime no longer relies on that low-level consume-and-refill sequence directly.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- After the recent bootstrap ordering change, runtime now does consume -> adjudicate -> refill-on-success.
- The old debug helper was still bypassing that path by consuming and refilling first, then materializing separately.
- That drift would make pg/debug assertions less trustworthy exactly where the staged graph-search execution path is changing fastest.

Review focus:
- Whether the debug helper now reflects the same candidate-selection and refill ordering semantics as runtime tuple production
- Whether exposing the existing bootstrap materialization/head helpers at `pub(super)` is an acceptable narrow boundary
- Whether any other debug helpers still exercise pre-runtime bootstrap sequencing and should eventually be realigned too
