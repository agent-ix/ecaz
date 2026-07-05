# Manifest: Task 41 SPIRE Diagnostic Relation Guards

- head SHA: `5c5e2f45e8c0806caa0e64d387d7651f1a650e0b`
- packet/topic: `31162-c1-task41-spire-diagnostic-relation-guards`
- timestamp: `2026-05-17T00:38:02Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet changes SQL diagnostic relation-resource handling and uses static
  validation plus PG18 compile coverage.

## Artifacts

### baseline-before.log

- lane / fixture / storage format / rerank mode: unsafe-comment baseline before
  the code checkpoint; no fixture/storage/rerank.
- command source: prior Task 41 strategy packet baseline at
  `a7f1333440254310b53420992b619ad81aa35d25`.
- key cited result line: `entries: 4579`.

### baseline-after.log

- lane / fixture / storage format / rerank mode: unsafe-comment baseline after
  the code checkpoint; no fixture/storage/rerank.
- command used: `make unsafe-baseline-report`
- key cited result lines:
  - `entries: 4539`
  - `326 src/lib.rs`

### unsafe-comment-audit.log

- lane / fixture / storage format / rerank mode: unsafe-comment audit; no
  fixture/storage/rerank.
- command used: `bash scripts/check_unsafe_comments.sh`
- key cited result line: `exit status: 0`.

### fmt-check.log

- lane / fixture / storage format / rerank mode: formatting gate; no
  fixture/storage/rerank.
- command used: `make fmt-check`
- key cited result line: `exit status: 0`.

### git-diff-check.log

- lane / fixture / storage format / rerank mode: whitespace gate; no
  fixture/storage/rerank.
- command used: `git diff --check`
- key cited result line: `exit status: 0`.

### cargo-check-pg18.log

- lane / fixture / storage format / rerank mode: PG18 compile gate; no
  fixture/storage/rerank.
- command used: `cargo check --all-targets --no-default-features --features pg18,bench`
- key cited result line: `exit status: 0`.
