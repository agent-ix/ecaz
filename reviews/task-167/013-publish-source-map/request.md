# Task 167 checkpoint: initial-generation source-map refresh

The routed physical tombstone path initially tracked only incremental DML
rows. This checkpoint closes that correctness gap by rebuilding the
coordinator-side `(source_tid, vec_id)` map during physical-generation publish
recovery, including exact replay. The refresh reads the frozen source identity
column from the source heap, derives the stable vec_id, and replaces only the
map rows for the control index. VACUUM can therefore route deletes for both
bulk-built and incrementally inserted rows.

Validation:

- `cargo check --no-default-features --features pg18` — passed at
  `82a17f87f`.
- `git diff --check` — passed.

This is not a closeout request. The physical integration test and live
multinode drill still need execution, as do FR-083-AC-4 parity, insert
throughput A/B, and the required 10k/50k/100k suite artifacts.
