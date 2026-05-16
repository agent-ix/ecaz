# Artifact Manifest

Head SHA: `f386b5f900286489e3ae603d17990babfda27362`

Packet/topic: `31032-spire-remote-search-test-split`

Lane / fixture / storage format / rerank mode: test module split only; no SQL
fixture; no storage format; no rerank mode.

Timestamp: `2026-05-14T03:32:39Z`

Isolated one-index-per-table or shared-table surfaces: not applicable.

## Artifacts

- `strategy.md`
  - Purpose: analysis of the oversized remote-search test file and the
    strategy for splitting it without adding to shrink-list files.

- `check_content_equivalence.py`
  - Purpose: checker used to compare the old monolith from `HEAD~2` with the
    new ordered include files, after normalizing path-only `include_str!`
    changes and separator-only EOF blank trimming.

- `content-equivalence.log`
  - Command: `python review/31032-spire-remote-search-test-split/artifacts/check_content_equivalence.py`
  - Key result: `normalized_content_match= True`.

- `line-counts.log`
  - Command: `wc -l src/tests/mod.rs src/tests/remote_search/*.rs`
  - Key result: `src/tests/remote_search.rs` no longer exists; largest new
    file is `contracts.rs` at 2,864 lines; `src/tests/mod.rs` remains 24,517
    lines.

- `diff-stat.log`
  - Command: `git diff --stat HEAD~2 HEAD -- src/tests/mod.rs src/tests/remote_search.rs src/tests/remote_search`
  - Key result: old 12,246-line `remote_search.rs` deleted; ten concern files
    plus `remote_search/mod.rs` added.

- `git-diff-check.log`
  - Command: `git diff --check HEAD~2 HEAD -- src/tests/mod.rs src/tests/remote_search.rs src/tests/remote_search`
  - Key result: no output.

- `git-diff-check-packet.log`
  - Command: `git diff --check -- review/31032-spire-remote-search-test-split`
  - Key result: no output.

- `cargo-fmt-check.log`
  - Command: `cargo fmt --check`
  - Key result: passed with the repository's existing stable-rustfmt warnings
    about unstable import options.

- `cargo-test-remote-search-no-run.log`
  - Command: `cargo test -p ecaz test_ec_spire_remote_search_sql_scores_selected_leaf_pids --no-run`
  - Key result: compile-only test build passed.
