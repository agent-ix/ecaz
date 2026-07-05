## Feedback: ADR-030 v2 Runtime Format Gate

Read `validate_runtime_scan_format` in `src/am/scan.rs` and the test
`validate_runtime_scan_format_rejects_grouped_v2_metadata` in `src/lib.rs`.

### What's right

- Returning a `GraphStorageDescriptor` from validation is the right shape. The
  descriptor then flows through the rest of the scan path (packets 324-328 build on
  it). Validation is not "bool plus side-channel state" — it's "typed handle or
  error."
- Explicit error `ADR030_GROUPED_V2_SCAN_UNSUPPORTED` means an operator who builds a
  v2 index and then runs a query gets a clear message, not a generic planner error.
  Good incident-response UX.
- The rejection test exists at the pg_test level (`lib.rs`), which is the right layer
  — it validates that a built v2 index is rejected by the real scan entry point, not
  just an isolated validation helper.

### Concerns

1. **Rejection point is the only currently-validated semantic.** The descriptor shape
   includes fields like `code_len`, `binary_word_count`, `search_code_len`,
   `rerank_code_len`. Nothing yet checks these against the actual metadata values at
   scan open. If a v1 scalar index somehow has inconsistent `code_len` vs the tuple
   width it actually wrote, the scan would not catch it. Worth adding a compatibility
   check inside `validate_runtime_scan_format` that at least asserts metadata fields
   are self-consistent for v1 scalar too, while the seam is being reworked.

2. **Error observability.** Is `ADR030_GROUPED_V2_SCAN_UNSUPPORTED` logged at WARN or
   raised as an ERROR? For an experimental format, raising an error is correct (no
   silent wrong results) but the error should carry the format version and the ADR
   number, so the operator knows what to search for.

### Observation

This is the load-bearing safety packet: it's what allows 324-328 to do read-side
plumbing without risk, because any grouped-v2 index is rejected before the plumbing
is engaged. Do not let subsequent packets move the rejection point — only the scorer
packet should be permitted to remove it.
