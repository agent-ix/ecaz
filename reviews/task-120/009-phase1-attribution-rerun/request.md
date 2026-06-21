# Task 120 Phase 1 Attribution Rerun

Please review the diagnostic attribution fix and the rerun evidence for the
packet 007 reviewer finding. Packet 007's recall/latency/funnel numbers were
genuine, but the target-block attribution was false for flat indexes because
`ec_spire_index_scan_leaf_target_block_rank_snapshot` emitted
`not_found_in_routed_leaves` when the index had no leaf block summaries.

## Code Change Under Review

- `src/am/ec_spire/scan/candidates.rs`
  - adds a no-summary fallback for leaf target block rank snapshots
  - when `scored_ranges` is empty, the diagnostic now checks the validated scan
    candidate frontier and emits `target_no_block_summaries` for retained truth
    targets
  - missing targets still report `not_found_in_routed_leaves`
- `crates/ecaz-cli/src/commands/bench/spire_pipeline.rs`
  - treats `target_no_block_summaries` as block-selected for stage containment
  - classifies retained no-summary targets as `candidate_or_rerank_cap` when a
    later miss must be attributed
  - adds focused unit coverage for miss attribution and stage containment

This is scoped to the diagnostic SQL and `bench spire-pipeline` attribution
logic. It does not change the normal scan executor's candidate selection path.

## Evidence

- Artifact manifest:
  `reviews/task-120/009-phase1-attribution-rerun/artifacts/manifest.md`
- Code head:
  `4617b0f3245e3c6ccdf13799a18912b0371ca4c9`
- Release backend check:
  `reviews/task-120/009-phase1-attribution-rerun/artifacts/backend-profile.log`
  reports `ecaz_build_profile = release`
- Host precheck:
  `reviews/task-120/009-phase1-attribution-rerun/artifacts/precheck-host.log`
  reports PostgreSQL `18.3`, `leaf_block_rows=0`,
  `leaf_block_summary_representatives=2`
- Suite status:
  `reviews/task-120/009-phase1-attribution-rerun/artifacts/suite-status.log`
  reports `completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Compact diagnostic summaries:
  - `reviews/task-120/009-phase1-attribution-rerun/artifacts/pipeline-stage-containment-summary.txt`
  - `reviews/task-120/009-phase1-attribution-rerun/artifacts/pipeline-target-block-rank-summary.txt`
- Focused tests:
  - `reviews/task-120/009-phase1-attribution-rerun/artifacts/cargo-test-miss-attribution.log`
  - `reviews/task-120/009-phase1-attribution-rerun/artifacts/cargo-test-stage-containment.log`

## Corrected Attribution Result

For each scale and nprobe below, `topology_route_set`,
`selected_leaf_blocks`, and `local_candidate_frontier` now have identical
contained/missing truth counts. In other words, the rerun no longer invents
block pruning on a flat index with no block summaries.

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

The corresponding target-block rows are either
`target_no_block_summaries` or true `not_found_in_routed_leaves` misses:

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

## Recall And Latency Reference

The rerun used 200 queries per scale and `nprobe=8,16,24,32`.

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

## Notes

- The suite uses task-local prefixes with `--allow-manifest-mismatch` because
  the staged manifests refer to the canonical `ec_real_*` prefixes.
- The raw per-query pipeline JSONLs from the rerun were pruned after reviewer
  feedback; the packet now keeps compact aggregate summaries for the cited
  diagnostic cells.
- The packet uses `--truth-corpus-file`; it does not generate or commit
  `truth-cache/`.
- Corpus/query TSV inputs remain outside git per repository packet rules.
- This packet is not Task 120 closeout. It corrects the Phase 1 attribution
  evidence so the next coarse/rerank measurement step is based on the right
  failure stage.
