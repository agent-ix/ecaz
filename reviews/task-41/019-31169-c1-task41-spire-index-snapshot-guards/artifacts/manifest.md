# Manifest: Task 41 SPIRE Index Snapshot Guards

- head SHA: `70f0614ee08008835a261fae4dde999e13e6a1ed`
- packet/topic: `31169-c1-task41-spire-index-snapshot-guards`
- timestamp: `2026-05-17T02:11:16Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet changes SQL diagnostic relation-resource handling and uses static
  validation plus PG18 compile coverage.

## Artifacts

### unsafe-baseline-before.txt

- lane / fixture / storage format / rerank mode: unsafe-comment baseline before
  the code checkpoint; no fixture/storage/rerank.
- command used: `git show 70f0614e^:scripts/unsafe_comment_baseline.txt`
- key cited result: source baseline for `baseline-before.log`.

### baseline-before.log

- lane / fixture / storage format / rerank mode: unsafe-comment baseline before
  the code checkpoint; no fixture/storage/rerank.
- command used: `bash scripts/unsafe_baseline_report.sh review/31169-c1-task41-spire-index-snapshot-guards/artifacts/unsafe-baseline-before.txt`
- key cited result lines:
  - `entries: 4447`
  - `234 src/lib.rs`

### baseline-after.log

- lane / fixture / storage format / rerank mode: unsafe-comment baseline after
  the code checkpoint; no fixture/storage/rerank.
- command used: `bash scripts/unsafe_baseline_report.sh`
- key cited result lines:
  - `entries: 4427`
  - `214 src/lib.rs`

### unsafe-comment-audit.log

- lane / fixture / storage format / rerank mode: unsafe-comment audit; no
  fixture/storage/rerank.
- command used: `bash scripts/check_unsafe_comments.sh`
- key cited result line: `exit status: 0`.

### fmt-check.log

- lane / fixture / storage format / rerank mode: formatting gate; no
  fixture/storage/rerank.
- command used: `make fmt-check`
- key cited result line: `cargo fmt --all -- --check`.
- notes: stable rustfmt emitted the existing warnings about unstable
  `imports_granularity` and `group_imports` settings.

### git-diff-check.log

- lane / fixture / storage format / rerank mode: whitespace gate; no
  fixture/storage/rerank.
- command used: `git diff --check 70f0614e^ 70f0614e`
- key cited result line: `exit status: 0`.

### cargo-check-pg18.log

- lane / fixture / storage format / rerank mode: PG18 compile gate; no
  fixture/storage/rerank.
- command used: `cargo check --all-targets --no-default-features --features pg18,bench`
- key cited result line: `Finished dev profile`.
- notes: existing warnings remain from PostgreSQL headers and unused imports in
  `src/am/mod.rs`.
