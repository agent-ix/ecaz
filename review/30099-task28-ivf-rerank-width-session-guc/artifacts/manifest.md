# Artifact Manifest

Packet: `review/30099-task28-ivf-rerank-width-session-guc`

This packet has no benchmark measurement artifacts. It is a code/test checkpoint for query-time IVF rerank-width control.

## Validation

- Head SHA: `d6a90fb`
- Command: `cargo pgrx test pg18 test_ec_ivf_heap_f32_rerank_width_bounds_exact_frontier`
- Result: passed

- Head SHA: `d6a90fb`
- Command: `cargo pgrx test pg18 test_ec_ivf_admin_snapshot`
- Result: passed

- Head SHA: `d6a90fb`
- Command: `git diff --check`
- Result: passed
