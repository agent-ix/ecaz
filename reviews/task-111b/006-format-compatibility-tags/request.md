# Task 111b Review Request: Format Compatibility and Tags

- Code commit: `2cabe4fdda00144dfa93747883579e05530fa98e`
- Packet: `reviews/task-111b/006-format-compatibility-tags`
- Task: `plan/tasks/111b-ivf-columnar-frozen-list-format.md`

## Summary

This checkpoint closes the Task 111b old-format compatibility and Task 42 tag-recording requirement.

It adds PG18 coverage for the aligned dense posting layout (`0x28`) by building an index with `dense_posting_blocks = 1, dense_posting_typed_layout = 1` and scanning it through the normal debug gettuple path. Existing focused fixtures cover row postings (`0x23`) and legacy dense blocks (`0x25`); all three are captured in this packet.

It also updates `docs/on-disk-format.md` with the IVF posting tuple tag set:

- row postings `0x23`;
- dense block `0x25`;
- abandoned packed segment tags `0x26` / `0x27`, retained only behind explicit experimental reloptions until cleanup;
- aligned dense block `0x28`;
- columnar frozen-list header `0x29`, version `1`, gated by `columnar_frozen_lists = 1`.

The metadata format version remains unchanged for Task 111b: IVF currently writes metadata version `2` and accepts versions `1..=2`.

## Validation

See `artifacts/manifest.md`.

- `cargo test -q test_ec_ivf_insert_vacuum_scan_safety --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
- `cargo test -q test_ec_ivf_dense_posting_blocks_scan_build_rows --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
- `cargo test -q test_ec_ivf_dense_typed_posting_blocks_scan_build_rows --lib`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 2126 filtered out`
