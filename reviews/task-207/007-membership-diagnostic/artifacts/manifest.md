# Artifact manifest

- Task bucket: `reviews/task-207/007-membership-diagnostic/`
- Validation date: 2026-08-04.
- Capture code checkpoint: `8eea5f965`; the later single-owner digest fix is
  `482a34e56`. The structured result rows report extension source SHA
  `534e37299`, while the packet-local head-membership capture is from the
  capture checkpoint above.
- Runner: `ecaz bench suite` with `task207-membership-10k.json`.
- Topology: 3 local PostgreSQL nodes, four build shards, 10k staged corpus.
- Arms: `stitched_bfs` and `partition_union`, one 10k step each.
- Head cap/search: 4096 / 128; queries: 200; benchmark iterations: 1.
- Results: `run/results.jsonl` and `run/suite-manifest.json`.
- Compact cited result lines: `run/stitched/distann-multinode-summary.log` and
  `run/union/distann-multinode-summary.log`.
- Membership evidence: `run/stitched/physical-head-membership.json` and
  `run/union/physical-head-membership.json`.
- Prediction evidence: `run/stitched/physical-stitched-persisted-predictions.json`
  and `run/union/physical-union-persisted-predictions.json`.
- Analysis: `membership-analysis.md`.
- Corpus: `ec_real_10k`; corpus/query data is not committed. The result rows
  record the corpus prefix and query digest.
- Run directories were outside the repository under
  `~/.ecaz/clusters/task207-membership-10k-*` and are removed after capture.
