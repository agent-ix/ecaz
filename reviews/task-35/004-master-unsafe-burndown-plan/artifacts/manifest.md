# Artifact Manifest: Master Unsafe Burndown Execution Plan

Head SHA: `7732bb4d09272105e4b95ebb033db65570cb487b`

Task bucket: `reviews/task-35`

Packet path: `reviews/task-35/004-master-unsafe-burndown-plan`

Timestamp: `2026-05-19T03:02:42Z`

Lane / fixture / storage format / rerank mode: not applicable; planning and
static audit packet.

Artifacts:

- `unsafe-baseline-report.log`
  - command: `make unsafe-baseline-report`
  - key result: baseline has 3,686 entries across 106 files; largest buckets
    are `src/am` with 2,936 entries, `src/tests` with 499, root `src` files
    with 177, and `src/quant` with 74.
- `baseline-by-subsystem.log`
  - command: `awk -F'[:/]' '{if ($1=="src" && $2=="am") bucket=$1"/"$2"/"$3; else if ($1=="src") bucket=$1"/"$2; else bucket=$1; count[bucket]++} END {for (b in count) print count[b], b}' scripts/unsafe_comment_baseline.txt | sort -nr`
  - key result: `src/am/ec_hnsw` 1,333; `src/am/ec_spire` 892;
    `src/tests` 499; `src/am/ec_ivf` 332; `src/am/ec_diskann` 237.
- `baseline-by-spire-area.log`
  - command: `awk -F'[:/]' '{if ($1=="src" && $2=="am" && $3=="ec_spire") bucket=$1"/"$2"/"$3"/"$4; else if ($1=="src" && $2=="am") bucket=$1"/"$2"/"$3; else if ($1=="src") bucket=$1"/"$2; else bucket=$1; count[bucket]++} END {for (b in count) print count[b], b}' scripts/unsafe_comment_baseline.txt | sort -nr`
  - key result: SPIRE splits into coordinator 294, DML frontdoor 168,
    CustomScan 122, storage 65, page 61, build 46, scan 37, vacuum 34.
- `baseline-by-file.log`
  - command: `awk -F: '{count[$1]++} ...' scripts/unsafe_comment_baseline.txt | sort -nr`
  - key result: largest files are `src/am/ec_hnsw/scan_debug.rs` 357,
    `src/am/ec_hnsw/scan.rs` 258, `src/am/ec_hnsw/build_parallel.rs` 201,
    `src/lib.rs` 177, and `src/am/ec_spire/dml_frontdoor/mod.rs` 159.
- `unsafe-block-count.log`
  - command: `rg -n 'unsafe\s*\{' src | wc -l`
  - key result: 3,778 unsafe blocks under `src`.
- `safety-comment-count.log`
  - command: `rg -n '// SAFETY:' src | wc -l`
  - key result: 138 SAFETY comments under `src`.
- `unsafe-declarations.log`
  - command: `rg -n '^\s*(pub\([^)]*\)\s+|pub\s+|pub\(crate\)\s+)?unsafe\s+fn\b|unsafe\s+extern|unsafe\s+impl\b|#\[unsafe' src`
  - key result: declaration-level audit input for the post-baseline cleanup
    pass.
- `audit-unsafe.log`
  - command: `bash scripts/check_unsafe_comments.sh`
  - key result: failed because current missing unsafe block line numbers are
    not all present in `scripts/unsafe_comment_baseline.txt`.
- `current-missing-unsafe-lines.txt`
  - command: local reproduction of `scripts/check_unsafe_comments.sh` missing
    line calculation without updating the baseline file:
    `while read rg unsafe line; test previous three lines for // SAFETY:; print
    file:line when missing; sort -u`
  - key result: 3,657 current missing-SAFETY entries across 107 files.
- `current-missing-by-subsystem.log`
  - command: `awk -F'[:/]' '{if ($1=="src" && $2=="am") bucket=$1"/"$2"/"$3; else if ($1=="src") bucket=$1"/"$2; else bucket=$1; count[bucket]++} END {for (b in count) print count[b], b}' current-missing-unsafe-lines.txt | sort -nr`
  - key result: `src/am/ec_hnsw` 1,299; `src/am/ec_spire` 886;
    `src/tests` 499; `src/am/ec_ivf` 326; `src/am/ec_diskann` 230.
- `current-missing-by-file.log`
  - command: `awk -F: '{count[$1]++} ...' current-missing-unsafe-lines.txt | sort -nr`
  - key result: largest current files are `src/am/ec_hnsw/scan_debug.rs` 354,
    `src/am/ec_hnsw/scan.rs` 258, `src/am/ec_hnsw/build_parallel.rs` 203,
    `src/lib.rs` 181, and `src/am/ec_spire/dml_frontdoor/mod.rs` 159.
- `current-missing-not-in-baseline.txt`
  - command: `comm -23 current-missing-unsafe-lines.txt scripts/unsafe_comment_baseline.txt`
  - key result: 1,596 current entries are absent from the line-number baseline.
- `baseline-entries-not-current.txt`
  - command: `comm -13 current-missing-unsafe-lines.txt scripts/unsafe_comment_baseline.txt`
  - key result: 1,625 baseline entries no longer correspond to current
    missing-SAFETY lines.
- `git-diff-check.log`
  - command: `git diff --check`
  - key result: passed with no output.

Isolated one-index-per-table or shared-table surfaces: not applicable.
