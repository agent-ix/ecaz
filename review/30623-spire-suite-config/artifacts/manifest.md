# Artifact Manifest

## `suite-manifest.json`

- head SHA: `7d7bd80c`
- packet/topic: `review/30623-spire-suite-config`
- lane: `task30 SPIRE real10k suite dry-run`
- fixture: `crates/ecaz-cli/suites/task30-spire-real10k.json`
- storage format: `turboquant`
- rerank mode: `rerank_width=25`
- isolated one-index-per-table or shared-table surfaces: shared corpus-prefix
  suite surface; dry-run only
- command: `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench suite run --config crates/ecaz-cli/suites/task30-spire-real10k.json --dry-run --manifest-output review/30623-spire-suite-config/artifacts/suite-manifest.json`
- timestamp: `2026-05-08`
- key result lines:
  - suite: `task30-spire-real10k`
  - dry_run: `true`
  - selected steps: `load-real10k-turboquant-n32-w25`, `storage-real10k-n32`,
    `explain-real10k-n32-p24-w25`, `latency-real10k-nprobe-sweep-w25`,
    `recall-real10k-nprobe-sweep-w25`
