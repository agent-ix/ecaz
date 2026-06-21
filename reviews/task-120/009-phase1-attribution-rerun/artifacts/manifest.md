# Task 120 Packet 009 Artifact Manifest

- head SHA: `4617b0f3245e3c6ccdf13799a18912b0371ca4c9`
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/009-phase1-attribution-rerun/`
- lane: local PG18 Intel host
- database/socket: `tqvector_bench_task120` on `/home/peter/.pgrx`, port `28818`
- fixture: staged real corpus at `data/staged-current/ec_real_{10k,50k,100k}_*.tsv`
- storage format: `ec_spire`, 4-bit vectors, flat leaf-block surface with
  `ec_spire.leaf_block_rows=0` and `ec_spire.leaf_block_summary_representatives=2`
- rerank mode: exact source rerank through `bench spire-pipeline`
- surfaces: isolated one-index-per-table prefixes per scale
  (`task120_phase1_rerun_real{10k,50k,100k}_spire`), not shared-table
- corpus data: TSV corpus/query/truth inputs were not committed; the suite reads
  staged local data directly and records the staged manifest paths in the
  command lines

## Build And Host Checks

- `cargo-build-ecaz-cli.log`
  - command: `cargo build -p ecaz-cli`
  - result: succeeded; one pre-existing `dead_code` warning for
    `crates/ecaz-cli/src/commands/corpus/load.rs:170`
- `cargo-pgrx-install-pg18-release.log`
  - command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`
  - result: succeeded and installed `ecaz.so` into the PG18.3 pgrx tree
- `backend-profile.log`
  - command: `target/debug/ecaz dev sql --pg 18 --db tqvector_bench_task120 --socket-dir /home/peter/.pgrx --raw --sql "SELECT ecaz_build_profile();"`
  - key result: `release`
- `precheck-host.log`
  - command: recorded by `suite-status.log` as `dev sql --pg 18 ...`
  - key result: PostgreSQL `18.3`, `leaf_block_rows=0`,
    `leaf_block_summary_representatives=2`
- `cargo-test-miss-attribution.log`
  - command: `cargo test -p ecaz-cli miss_attribution_classifies_hit_routing_block_and_cap_misses`
  - key result: `1 passed; 0 failed`
- `cargo-test-stage-containment.log`
  - command: `cargo test -p ecaz-cli stage_containment_records_per_stage_truth_retention`
  - key result: `1 passed; 0 failed`

## Suite Runner

- `suite.json`
  - checked-in task-local `SuiteConfig` for the packet
  - steps: precheck host, load 10k/50k/100k, run `bench spire-pipeline` for
    10k/50k/100k
  - sweep: `nprobe=8,16,24,32`
  - query limit: 200 queries per scale
  - recall source: `--truth-corpus-file data/staged-current/ec_real_*_corpus.tsv`
    rather than `truth-cache/`
- `suite-manifest.json`
  - final non-dry-run suite manifest
- `suite-results.jsonl`
  - structured per-step result records
- `suite-status.log`
  - command: `target/debug/ecaz bench suite status reviews/task-120/009-phase1-attribution-rerun/artifacts/suite.json --artifact-dir reviews/task-120/009-phase1-attribution-rerun/artifacts`
  - key result: `completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `suite-report.md`, `suite-report-results.jsonl`
  - command: `target/debug/ecaz bench suite report ...`
  - key result: report emitted after the final rerun with rebuilt CLI attribution
    handling

Historical dry-run/audit artifacts in this packet:

- `suite-dry-run.log`
- `suite-manifest.dry-run.json`
- `suite-audit.log`

These record the pre-run/audit pass only; the cited evidence below uses the
final non-dry-run suite artifacts.

## Per-Scale Artifacts

For each scale, the suite emitted:

- `load-{10k,50k,100k}-spire.log`
- `pipeline-{10k,50k,100k}-spire.log`
- `pipeline-{10k,50k,100k}-funnel.jsonl`
- `pipeline-{10k,50k,100k}-stage-containment.jsonl`
- `pipeline-{10k,50k,100k}-leaf-block-rank.jsonl`
- `pipeline-{10k,50k,100k}-target-block-rank.jsonl`
- `pipeline-{10k,50k,100k}-target-candidate-rank.jsonl`

## Target Block Attribution Counts

`target_no_block_summaries` means the truth target was found in the routed leaf
candidate frontier for a flat index that has no leaf block summaries. These rows
replace the packet 007 false `not_found_in_routed_leaves` attribution for those
targets.

| Scale | nprobe | target_no_block_summaries | not_found_in_routed_leaves |
| --- | ---: | ---: | ---: |
| 10k | 8 | 1984 | 16 |
| 10k | 16 | 1997 | 3 |
| 10k | 24 | 2000 | 0 |
| 10k | 32 | 2000 | 0 |
| 50k | 8 | 1710 | 290 |
| 50k | 16 | 1853 | 147 |
| 50k | 24 | 1896 | 104 |
| 50k | 32 | 1925 | 75 |
| 100k | 8 | 1539 | 461 |
| 100k | 16 | 1699 | 301 |
| 100k | 24 | 1788 | 212 |
| 100k | 32 | 1841 | 159 |

## Corrected Stage Containment

For each row below, `topology_route_set`, `selected_leaf_blocks`, and
`local_candidate_frontier` all produced the same contained/missing truth counts.
That confirms the flat-index attribution rerun no longer reports block-level
loss when no block summaries exist.

| Scale | nprobe | contained | missing |
| --- | ---: | ---: | ---: |
| 10k | 8 | 1984 | 16 |
| 10k | 16 | 1997 | 3 |
| 10k | 24 | 2000 | 0 |
| 10k | 32 | 2000 | 0 |
| 50k | 8 | 1710 | 290 |
| 50k | 16 | 1853 | 147 |
| 50k | 24 | 1896 | 104 |
| 50k | 32 | 1925 | 75 |
| 100k | 8 | 1539 | 461 |
| 100k | 16 | 1699 | 301 |
| 100k | 24 | 1788 | 212 |
| 100k | 32 | 1841 | 159 |

## Recall And Latency

| Scale | nprobe | recall@k | p50 | p95 |
| --- | ---: | ---: | ---: | ---: |
| 10k | 8 | 0.9920 | 37.642 ms | 52.469 ms |
| 10k | 16 | 0.9985 | 67.859 ms | 76.061 ms |
| 10k | 24 | 1.0000 | 95.599 ms | 105.180 ms |
| 10k | 32 | 1.0000 | 120.342 ms | 141.071 ms |
| 50k | 8 | 0.8550 | 73.666 ms | 90.426 ms |
| 50k | 16 | 0.9265 | 141.136 ms | 183.923 ms |
| 50k | 24 | 0.9480 | 206.904 ms | 242.565 ms |
| 50k | 32 | 0.9625 | 277.711 ms | 317.519 ms |
| 100k | 8 | 0.7695 | 112.308 ms | 148.634 ms |
| 100k | 16 | 0.8495 | 208.315 ms | 255.097 ms |
| 100k | 24 | 0.8940 | 309.146 ms | 384.287 ms |
| 100k | 32 | 0.9205 | 436.052 ms | 519.315 ms |

## Suite Deviations And Guardrails

- The load steps use task-local prefixes with `--allow-manifest-mismatch`
  because the staged manifest names the canonical `ec_real_*` prefix while this
  packet isolates its tables under `task120_phase1_rerun_*`.
- This packet intentionally does not commit corpus/query TSV files or any
  `truth-cache/` directory.
- This packet is an attribution rerun, not Task 120 closeout. It does not claim
  storage improvement or merge readiness for a query-path change.
