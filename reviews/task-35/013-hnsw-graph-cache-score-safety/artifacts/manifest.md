# Artifact Manifest: HNSW Graph Cache and Grouped Score Safety

Head SHA: `68d4bba75018bad2f1f10f5c5a673cdabe686189`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/013-hnsw-graph-cache-score-safety/`

Slice scope: HNSW grouped traversal and rerank scoring path in
`src/am/ec_hnsw/scan.rs`; baseline entries for that file in
`scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a doc/refactor slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:08:45 -0700`
  - result: expected failure from line drift before updating the baseline.
- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:08:32 -0700`
  - key line: `entries: 3264`, `files: 102`.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:08:32 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:09:05 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3212 entries`.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:09:45 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update-after-fmt.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:10:36 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3212 entries`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:09:17 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:09:17 -0700`
  - key line: `entries: 3212`, `files: 102`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:11:11 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s)`.
  - result: passed; pre-existing unused-import warnings in `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:13:10 -0700`
  - result: passed with no output.
- `unsafe-audit-final.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:10:48 -0700`
  - result: passed with no output.
- `unsafe-baseline-final.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:10:48 -0700`
  - key line: `entries: 3212`, `files: 102`.

## Key result lines cited by `request.md`

- Baseline start: 3,264 entries, 102 files.
- Baseline end: 3,212 entries, 102 files.
- Net reduction: 52 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 193 entries.
- `src/am/ec_hnsw/scan.rs` end: 141 entries.
