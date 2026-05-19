# Artifact Manifest: HNSW Frontier Refill Safety

Head SHA: `5aacdb2b46f344187f0bf140fe12d979b632c8b9`

Task bucket: `reviews/task-35`
Packet path: `reviews/task-35/017-hnsw-frontier-refill-safety/`

Slice scope: HNSW frontier-head exact refinement, layer-0 refill/top-up, and
test/pg_test bootstrap refill callbacks in `src/am/ec_hnsw/scan.rs`; baseline
entries for that file in `scripts/unsafe_comment_baseline.txt`.

Surface: PG18, ec_hnsw access method. No corpus, no benchmark.
This is a doc-only slice; no isolated/shared-table choice applies.
Timestamps are local Pacific time from artifact mtimes.

## Artifacts

- `unsafe-baseline-before.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:33:04 -0700`
  - key line: `entries: 3131`, `files: 102`.
- `unsafe-audit-before-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:33:14 -0700`
  - result: expected failure from line drift before updating the baseline.
- `scan-rs-diff-before-baseline.patch`
  - command: `git diff -- src/am/ec_hnsw/scan.rs`
  - timestamp: `2026-05-18 21:33:04 -0700`
  - result: retained patch snapshot before baseline rewrite.
- `cargo-fmt.log`
  - command: `cargo fmt --all`
  - timestamp: `2026-05-18 21:33:24 -0700`
  - result: passed; only stable-toolchain warnings for unstable
    `rustfmt.toml` options.
- `unsafe-baseline-update.log`
  - command: `bash scripts/check_unsafe_comments.sh --update-baseline`
  - timestamp: `2026-05-18 21:33:43 -0700`
  - key line: `wrote scripts/unsafe_comment_baseline.txt with 3115 entries`.
- `unsafe-audit-after.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - timestamp: `2026-05-18 21:33:53 -0700`
  - result: passed with no output.
- `unsafe-baseline-after.log`
  - command: `make unsafe-baseline-report`
  - timestamp: `2026-05-18 21:33:53 -0700`
  - key line: `entries: 3115`, `files: 102`.
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - timestamp: `2026-05-18 21:34:08 -0700`
  - key line: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 15.05s`.
  - result: passed; pre-existing unused-import warnings in
    `src/am/common/parallel.rs` and `src/am/mod.rs`.
- `git-diff-check.log`
  - command: `git diff --check`
  - timestamp: `2026-05-18 21:34:24 -0700`
  - result: passed with no output.

## Key result lines cited by `request.md`

- Baseline start: 3,131 entries, 102 files.
- Baseline end: 3,115 entries, 102 files.
- Net reduction: 16 baseline entries.
- `src/am/ec_hnsw/scan.rs` start: 60 entries.
- `src/am/ec_hnsw/scan.rs` end: 44 entries.
