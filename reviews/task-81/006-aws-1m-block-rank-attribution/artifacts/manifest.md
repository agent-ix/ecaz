# Task 81 Packet 006 Manifest: AWS 1M Block-Rank Attribution Attempt

- head SHA at packet creation: `79c3297ff`
- branch: `task-81-spire-leaf-block-summary-format`
- task bucket: `reviews/task-81/`
- packet: `reviews/task-81/006-aws-1m-block-rank-attribution/`
- timestamp: `2026-06-05`
- lane: AWS retained `1m`, database `postgres`
- profile: `ec_spire`
- runner: `ecaz cloud bench` driving `ecaz bench suite`
- storage format: retained SPIRE RaBitQ block16/tg256 surface
- purpose: diagnose why Task 81 packet 003/005 AWS q500 rows stayed at
  recall@10 `0.9832` despite local 100k recall improving with broader routing.

## Commands And Artifacts

- `suite-aws-1m-block-rank-attribution-q50.json`: checked-in SuiteConfig for
  a q50 rank-attribution run using `bench spire-pipeline` and
  `leaf_block_rank_output`.
- `artifacts/suite-audit-q50-rerun.log`: local suite audit. Key line:
  `audit passed: 2 steps`.
- `artifacts/ssm-register-leaf-block-rank-snapshot-rerun3.json`: successful
  remote catalog registration for
  `ec_spire_index_scan_leaf_block_rank_snapshot(oid, real[], bigint[])`.
- `artifacts/ssm-rank-offset-probe-q1-rerun-final4.json`: q1 offset probe.
  Key output:
  - `offset0|block_ranked|8|3|132|8944|0`
  - `offset0|not_found_in_routed_leaves|2|0|||2`
  - `offset1|block_ranked|10|10|2|266|0`
- `artifacts/ssm-create-remote-q50-truth-cache.json`: remote q50 truth-cache
  slice creation from the q500 cache. The file existed and reported q50 shape,
  but it intentionally failed `bench recall` descriptor validation because the
  query hash must cover the limited q50 query vector set.
- `artifacts/ssm-cloud-bench-rank-attribution-q50-rerun2-fail.json`: q50
  cached attempt failed with `truth cache file ... does not match query set and
  k`.
- `artifacts/cloud-bench-rank-attribution-q50-nocache.log`: q50 no-cache
  attempt. It was cancelled after remote process inspection showed it was
  fetching/scoring the full 1M corpus for exact truth.
- `artifacts/ssm-progress-during-q50-nocache.json`: remote progress check for
  the no-cache attempt. Key line: active query
  `SELECT id, source FROM task67_1m_hnsw_m7g2xlarge_corpus ORDER BY id`.
- `suite-aws-1m-block-rank-attribution-raw-q50.json`: raw SQL fallback suite
  that used q500 truth IDs directly with the confirmed `+1` local sequence
  offset. This was also cancelled after running longer than useful for a q50
  diagnostic.
- `artifacts/cloud-pause-after-cancelled-attribution.log`: AWS pause request
  after cancelling the rank-attribution attempts.

## Findings

- The correct mapping from q500 truth IDs to SPIRE local sequence targets on
  this retained surface is `truth_id + 1`.
- The first query is not representative of the AWS recall miss: with offset
  `+1`, all 10 exact targets were routed and selected under the global cap,
  with block ranks `2..266`.
- The q50 diagnostic path was not cost-effective through the current suite
  interfaces: cache slicing failed descriptor validation, and uncached q50
  truth computation streamed the full 1M corpus.
- This packet does not close Task 81 and does not provide an accepted AWS row.
  It is retained as negative provenance before switching the next packet to the
  corrected Task 79 comparison baseline requested by the user.
