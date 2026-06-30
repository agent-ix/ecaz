# Task 124 Phase 6 Evidence Correction Manifest

- head SHA before packet: `ba1cd054bfaca9f0ed39b290392fe754708432cb`
- task bucket: `reviews/task-124`
- packet path: `reviews/task-124/019-phase6-evidence-correction`
- lane: reviewer feedback response / evidence correction
- date: 2026-06-29
- timestamp: 2026-06-30T01:02:57Z

## Feedback Processed

- Source: `reviews/task-124/016-closeout-shelve/feedback/2026-06-29-01-reviewer.md`
- Required correction: stop citing packet 015 as completed cold-cache /
  IO-sensitive evidence because the macOS `F_NOCACHE` helper affected only the
  transient file descriptors it opened, not PostgreSQL's relation reads.

## Files Changed

- `plan/tasks/124-ivf-tq-stage2-rerank-pipeline.md`: Phase 6 now records packet
  015 as an attempted local run, not controlled cold-cache evidence.
- `reviews/task-124/016-closeout-shelve/request.md`: Phase 6 audit and closeout
  wording corrected.
- `reviews/task-124/016-closeout-shelve/artifacts/manifest.md`: evidence source,
  criteria, and key-fact wording corrected.
- `reviews/task-124/017-speed-objective-correction/request.md`: packet 015
  caveat added.
- `reviews/task-124/017-speed-objective-correction/artifacts/manifest.md`:
  packet 015 caveat added.
- `crates/ecaz-cli/src/commands/dev/relation_cache.rs`: non-Linux non-dry-run
  eviction now fails instead of reporting macOS `F_NOCACHE` as an eviction mode.

## Validation Commands

- command: `cargo fmt --check`
  - result: passed
  - note: rustfmt emitted existing stable-channel warnings for unstable
    import-formatting options.
- command: `cargo test -p ecaz-cli relation_file_match_includes_segments_and_forks`
  - result: passed
  - key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 411 filtered out`
- command: `cargo check -p ecaz-cli`
  - result: passed
  - key line: `Finished dev profile`
  - warning: unrelated existing dead-code warning for
    `LoadedDistributedPlacementConfig::path`

## Outcome

The reviewer correction is incorporated. Packet 015 should not be used as
Phase 6 cold-cache evidence, and future macOS runs cannot accidentally report
`F_NOCACHE` as a successful PostgreSQL relation-cache eviction.
