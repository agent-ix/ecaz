## Feedback: PqFastScan Scan-Analysis Debug Surface Rename

Read the canonical vs alias scan-analysis helpers in `src/lib.rs`,
the shared `pq_fastscan_scan_*_values(...)` value helpers, and
`validate_debug_index(...)`.

### What's right

- **Same architectural shape as 394: shared value helpers under
  canonical and legacy wrappers.** That prevents the two surfaces
  from drifting in what they compute, even as column names differ.
- **`grouped_result_count` → `pq_fastscan_result_count` only on
  the canonical helper.** Keeps old consumers working while new
  ones see product-name columns.
- **Shared `validate_debug_index(...)` centralizes pre-flight
  checks.** Good incidental cleanup — the previous duplicated
  validation across five helpers was exactly the kind of
  copy-paste that rots.

### Concerns

1. **Five pairs of canonical/legacy helpers is a lot of
   compatibility surface.** Each pair is a single-line wrapper
   today, but five is the point where "we'll clean them up
   later" starts to mean "we never will." Worth committing to a
   removal-by-task-N milestone in the ADR-032 followups. If the
   answer is "never, these are public SQL," then own that and
   document the legacy surface as stable-API.
2. **Only clippy validation this slice.** The packet notes pass
   for `cargo clippy` but not `cargo check --tests`. Likely
   fine for SQL-surface renaming (nothing structural changed),
   but worth confirming the test-compile lane was green.
3. **Linker gap.** Standard for this arc.

### Observation

Consistent with 394. Together they move the whole analysis debug
surface to canonical naming with conservative back-compat.
