# Review Request: SPIRE Phase 13e Leaf Assignment Export

Task: Task 30 Phase 13e
Code commit: `f671da96423a6863a723262a8f7da3c78ed786d5`

## Summary

This checkpoint adds the first production operator surface for leaf-owned
remote materialization:

- New SQL function `ec_spire_index_leaf_base_assignment_snapshot(index_oid,
  leaf_pids)` returns active coordinator leaf base assignment rows for either
  selected leaf PIDs or all available leaves when the array is empty.
- Returned rows include `leaf_pid`, `object_version`, `row_index`,
  `assignment_flags`, `vec_id`, opaque `row_locator`, `heap_block`,
  `heap_offset`, `heap_ctid`, `payload_format`, `gamma`, and the encoded
  assignment payload.
- New SQL script
  `scripts/spire-aws/export-coordinator-leaf-base-assignments.sql` exports the
  coordinator-owned leaf rows assigned to one remote node.
- `scripts/spire-aws/register.sh` now writes
  `remote-leaf-materialization/node-*-coordinator-base-assignments.tsv` during
  registration, giving the AWS flow durable per-node materialization input
  instead of only row-hash shard files.

## Validation

Artifacts are under
`reviews/task-30/967-spire-phase13e-leaf-assignment-export/artifacts/`.

- `cargo check -p ecaz --lib` passed.
- `bash -n scripts/spire-aws/register.sh` passed.
- `cargo fmt --all -- --check` passed with existing stable-rustfmt warnings for
  ignored nightly-only import options.
- `git diff --check HEAD` passed.

## Scope Notes

This exports coordinator leaf base assignments; it does not yet build matching
remote leaf objects from that export. Delta assignment export and the remote
writer/materializer remain follow-up work before distributed reads can be
declared valid.
