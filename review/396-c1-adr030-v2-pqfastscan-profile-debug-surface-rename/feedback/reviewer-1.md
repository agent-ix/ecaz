## Feedback: PqFastScan Profile Debug Surface Rename

Read the canonical profile helpers and the shared
`debug_scan_hot_path_profile_values` / `pq_fastscan_rerank_profile_values`
extraction helpers in `src/lib.rs`.

### What's right

- **Finishes the debug-surface sweep started in 394 and 395.**
  After this, the rerank profile and hot-path profile are both
  available under canonical `pq_fastscan_*` names with the legacy
  wrappers preserved.
- **Two new pg tests exercise the canonical helpers *and*
  canonical env names together.** That's the integration surface
  operators will actually hit — helper + env + column names all
  at once. Better than testing each in isolation.
- **Generic `tqhnsw_debug_scan_hot_path_profile()` intentionally
  left with legacy column names.** That's the right call — it's
  a shared helper that happens to surface grouped counters, not a
  grouped-specific helper. Renaming it would overpromise.

### Concerns

1. **Two profile helpers for the same underlying data.** Same
   concern as 395: compatibility surface accumulates. The shared
   value helpers bound the risk but the double-surface is real.
2. **No explicit test that canonical and legacy helpers return
   identical numbers.** The shared value helpers make this
   near-guaranteed by construction, but a one-line cross-helper
   equivalence test would catch future refactors that accidentally
   split the implementations.
3. **Linker gap.** Two new pg tests did not run locally.

### Observation

Three packets (394+395+396) in, the debug surface is consistent
with the product name and the compatibility aliases all share
source-of-truth extractors. Mission accomplished for this cluster
of cleanup slices.
