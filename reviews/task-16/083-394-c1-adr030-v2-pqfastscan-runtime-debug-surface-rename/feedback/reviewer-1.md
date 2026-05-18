## Feedback: PqFastScan Runtime Debug Surface Rename

Read `PqFastScanRuntimeSettings`, `current_pq_fastscan_runtime_settings`,
and the canonical vs alias debug-helper pair in `src/lib.rs`.

### What's right

- **Shared settings struct prevents drift.** Both the canonical
  `tqhnsw_debug_pq_fastscan_runtime_settings()` and the
  compatibility `tqhnsw_debug_adr030_runtime_settings()` helper
  now consume one struct. If the underlying lookup logic changes,
  both surfaces update in lockstep by construction.
- **Column renames on canonical surface only.** Old helper keeps
  `grouped_*` column names for compatibility; canonical helper
  emits `pq_fastscan_*`. That's the right split — external
  dashboards pinned to `grouped_scan_window` don't break, while
  new consumers see canonical names.
- **Test moved onto the canonical surface.** The regression
  coverage now validates the intended new naming, not the legacy
  one. Legacy still works as documented; future regressions would
  be caught on the canonical path first.

### Concerns

1. **Two debug helpers with overlapping semantics.** Increases the
   test-maintenance surface and the chance of silent divergence if
   a future change goes through only one path. The shared
   settings struct mitigates it but doesn't eliminate it — helper
   signatures/column counts could still drift independently.
   Worth a fixed-cardinality assertion (both helpers return N
   rows / N columns of the expected shape) as a meta-test.
2. **Linker gap.** Debug-surface changes are low-risk for
   correctness but high-risk for operator-facing workflows. The
   pg test did not run locally.

### Observation

Small but visible cleanup. The branch's debug surface now names
things the way the runtime does.
