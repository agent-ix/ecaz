# Task 131 Packet 004 Artifact Manifest

- Head SHA: `8d761e06a68a739428cde3b3ac81d6aa0c194e5f`
- Task bucket: `reviews/task-131/`
- Packet: `reviews/task-131/004-phase1-local-mi-ab-suite/`
- Timestamp: `2026-07-01T05:12:34Z`
- Lane: local PG18 multi-instance suite preparation.
- Fixture matrix: 10k, 50k, 100k; `n128/b4/nprobe96`; `n1024/b2/nprobe64`.
- Storage format: `rabitq`.
- Rerank mode: production-read-only, `top_k=10`, recall enabled against staged truth corpus.
- Surface isolation: one local multinode fixture per scale/surface, with one coordinator and three remotes.

## Artifacts

### `task131-phase1-local-mi-ab-suite.json`

- Purpose: FR-038 `ecaz bench suite` config for Task 131 Phase 1 local multi-instance A/B.
- Matrix:
  - 10k / 50k / 100k
  - `n128/b4` and `n1024/b2`
  - baseline `ec_spire.remote_search_global_pre_heap_merge=off`
  - candidate `ec_spire.remote_search_global_pre_heap_merge=on`
- Key detail: both variants use `timeline_payload=none` so the production-read timeline calls the no-payload remote heap path that the current global pre-heap implementation gates on.

### `suite-audit.log` and `suite-audit.stdout`

- Command: `target/debug/ecaz bench suite audit --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/suite-audit.log`
- Exit status: `0`
- Key result: `audit passed: 6 steps`

### `dryrun-manifest.json`, `dryrun-suite.log`, and `dryrun-suite.stdout`

- Command: `target/debug/ecaz bench suite run --config reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/task131-phase1-local-mi-ab-suite.json --dry-run --manifest-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/dryrun-manifest.json --results-output reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/dryrun-results.jsonl --log-file reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/dryrun-suite.log`
- Exit status: `0`
- Key result: dry-run expanded six `spire-local-multinode` commands.
- Key detail: every expanded command includes both:
  - `name=baseline;timeline_payload=none;guc=ec_spire.remote_search_global_pre_heap_merge=off`
  - `name=global-preheap-on;timeline_payload=none;guc=ec_spire.remote_search_global_pre_heap_merge=on`
- Note: dry-run mode did not produce `dryrun-results.jsonl` because no benchmark result rows are emitted without execution.

### `cargo-test-production-read.log`

- Command: `cargo test production_read --package ecaz-cli > reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/cargo-test-production-read.log 2>&1`
- Exit status: `0`
- Key result: `3 passed; 0 failed`

### `cargo-test-local-multinode-expansion.log`

- Command: `cargo test spire_local_multinode_step_expands_local_four_instance_lane --package ecaz-cli > reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/cargo-test-local-multinode-expansion.log 2>&1`
- Exit status: `0`
- Key result: `1 passed; 0 failed`

### `cargo-check-ecaz-cli.log`

- Command: `cargo check --package ecaz-cli > reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/cargo-check-ecaz-cli.log 2>&1`
- Exit status: `0`
- Key result: `Finished dev profile [unoptimized + debuginfo]`
- Note: the log includes the existing `LoadedDistributedPlacementConfig::path` dead-code warning.

### `git-diff-check-head.log`

- Command: `git diff --check HEAD~1..HEAD > reviews/task-131/004-phase1-local-mi-ab-suite/artifacts/git-diff-check-head.log 2>&1`
- Exit status: `0`
- Key result: no whitespace errors; artifact is empty.

## Cleanup Notes

- An initial real 10k `n128/b4` run was interrupted after dry-run inspection showed the nested command still used the tuple-payload timeline path.
- Generated distributed TSVs and the interrupted local PG data directory were removed before this packet was committed.
- This packet has no `*.tsv` or `*.tsv.gz` files.

