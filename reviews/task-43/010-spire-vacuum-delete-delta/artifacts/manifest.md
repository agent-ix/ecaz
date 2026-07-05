# Task 43 Packet 010 Artifact Manifest

Head SHA: `cc79787911a7aec2080c49af34e91ef4700c0af7`

Task bucket: `reviews/task-43/010-spire-vacuum-delete-delta`

Timestamp: `2026-05-18T13:18:05-07:00`

Surface: pure Rust SPIRE vacuum/delete-delta visibility and delta snapshot
validation. No PostgreSQL table, index, storage format fixture, or rerank
runtime table was created; isolated/shared table distinction is not applicable.

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `miri-spire-vacuum-visible-delete-delta.log` | `cargo +nightly miri test --lib miri_collect_visible_assignments_excludes_delete_delta_targets` | 1 passed; 0 failed; 1796 filtered; vacuum visible assignment filtering excludes delete-delta targets and boundary rows while keeping live base and delta-insert rows. |
| `miri-spire-delta-snapshot.log` | `cargo +nightly miri test --lib miri_delta_epoch_draft_from_snapshot` | 6 passed; 0 failed; 1791 filtered; delta publication plus unknown, mismatched, stale, duplicate, and already-deleted delete-target rejection. |
| `miri-spire-delta-replacement-fold.log` | `cargo +nightly miri test --lib miri_replacement_leaf_rows_fold_active_deltas_into_base_leaf_rows` | 1 passed; 0 failed; 1796 filtered; replacement fold excludes active delete-delta targets. |
| `cargo-fmt-check.log` | `cargo fmt --all -- --check` | Exit 0; rustfmt emitted existing unstable-option warnings. |
| `git-diff-check.log` | `git diff --check` | Exit 0; no whitespace errors. |

## Coverage Notes

- The new vacuum test uses `SpireLocalObjectStore`, real leaf/delta partition
  objects, `SpirePublishedEpochSnapshot`, `collect_delete_vec_ids_by_base_pid`,
  and `collect_visible_assignments`.
- The first Miri run caught a malformed test fixture where delete-delta rows
  used payload format 1. The fixture was corrected to the production
  delete-delta contract: tombstone/delete flags, payload format 0, gamma 0,
  and an empty payload.
- Existing delta snapshot tests were promoted with `miri_` prefixes instead of
  duplicating their coverage.
