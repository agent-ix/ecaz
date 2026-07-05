# Artifact Manifest: HNSW Linear Output Safety

Head SHA: `32d33db43fecb87a12a05cf3a0fa69fc4ddb9dcc`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/018-hnsw-linear-output-safety/`

Slice scope: HNSW graph traversal output refresh, linear fallback page scanning,
scan element scoring, and PostgreSQL scan output writes in
`src/am/ec_hnsw/scan.rs`; baseline entries for that file in
`scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a doc/refactor slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:37:36 -0700`
  - key line: `entries: 3115`, `files: 102`.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:37:47 -0700`
  - result: expected failure from line drift before updating the baseline.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:37:36 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:37:56 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:38:19 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3096 entries`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:38:25 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:38:25 -0700`
  - key line: `entries: 3096`, `files: 102`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:38:39 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 14.07s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:39:01 -0700`
  - result: passed with no output.

## Key result lines cited by `request.md`

- Baseline start: 3,115 entries, 102 files.
- Baseline end: 3,096 entries, 102 files.
- Net reduction: 19 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 44 entries.
- `src/am/ec_hnsw/scan.rs` end: 25 entries.
