# Task 124 Phase 6 Evidence Correction

## Summary

This packet addresses reviewer feedback on
`reviews/task-124/016-closeout-shelve/feedback/2026-06-29-01-reviewer.md`.

The correction is narrow:

- packet 015 is no longer cited as valid cold-cache / IO-sensitive evidence;
- packet 016, packet 017, and the task file now say packet 015 was an attempted
  local macOS run only;
- `ecaz dev evict-relation-cache` no longer reports macOS `F_NOCACHE` as a
  successful relation-cache eviction path.

Task 124 remains reopened for the TQ speed objective from packet 017. This
packet does not close the task and does not change the speed acceptance bar:
future TQ slices need TQ-before/TQ-after speed evidence.

## Code Change

`crates/ecaz-cli/src/commands/dev/relation_cache.rs` now only reports eviction
success on platforms using `posix_fadvise(DONTNEED)`. On macOS and other
non-Linux platforms, a non-dry-run eviction request fails with a message that
`F_NOCACHE` is per-fd and does not evict PostgreSQL's separate relation reads.

## Validation

- `cargo fmt --check`: passed; rustfmt emitted existing stable-channel warnings
  for unstable import-formatting options.
- `cargo test -p ecaz-cli relation_file_match_includes_segments_and_forks`:
  passed, 1 test.
- `cargo check -p ecaz-cli`: passed; emitted an unrelated existing dead-code
  warning for `LoadedDistributedPlacementConfig::path`.
