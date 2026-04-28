# Artifact Manifest

Packet: `review/30103-task28-ivf-vacuum-posting-page-compaction`

This packet has no benchmark measurement artifacts. It records a code/test checkpoint for page-local IVF posting compaction during vacuum.

## Validation

- Head SHA: `2c1196c2`
- Command: `cargo pgrx test pg18 test_ec_ivf_vacuum_compacts_deleted_posting_space_for_reuse`
- Result: passed

- Head SHA: `2c1196c2`
- Command: `cargo pgrx test pg18 test_ec_ivf_vacuum_bulkdelete_removes_dead_heap_tid`
- Result: passed

- Head SHA: `2c1196c2`
- Command: `cargo pgrx test pg18 test_ec_ivf_vacuum_repairs_empty_list_directory_refs`
- Result: passed

- Head SHA: `2c1196c2`
- Command: `git diff --check`
- Result: passed
