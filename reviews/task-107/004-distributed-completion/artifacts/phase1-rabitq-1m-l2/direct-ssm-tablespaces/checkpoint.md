# Cell Checkpoint: phase1-rabitq-1m-l2

Status: completed.

## Intent

- Checklist cell: `phase1-rabitq-1m-l2`.
- Phase: 1, single-node multi-disk / multi-store.
- Scale: 1m representative corpus (`ec_real_ann_benchmarks_anchor`, 990,000
  corpus rows and 10,000 query rows in prior packet evidence).
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 2.
- Store tablespaces: `ecaz_spire_store_1,ecaz_spire_store_2`.
- Prefix: `task107_phase1_rabitq_1m_l2`.
- Index: `task107_phase1_rabitq_1m_l2_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-1m-l2/direct-ssm-tablespaces/`.

## Execution Policy

Run this cell to completion or command failure. Do not stop the AWS instances
after the cell unless the user explicitly asks or a concrete cleanup failure
makes that unsafe.

## Planned Work

1. Use coordinator-local SSM to reuse/download the existing 1m representative
   corpus from S3:
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_corpus.tsv`
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_queries.tsv`
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_manifest.json`
2. Drop only `task107_phase1_rabitq_1m_l2%` residue before loading.
3. Build only `task107_phase1_rabitq_1m_l2_idx` with
   `local_store_count=2` and explicit
   `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`.
4. Run the packet-local `ecaz bench suite` config on the coordinator node using
   the node-local 1m corpus file for exact truth generation.
5. Capture storage evidence through the suite `storage` step.
6. Clean up only `task107_phase1_rabitq_1m_l2%` objects.

The SSM command was sent with a high AWS service timeout and
`AWS-RunShellScript` `executionTimeout=172800`; there is no benchmark time cap
in the payload.

## Result

- SSM command: `63bd3e6c-a375-4957-acde-a146a33dc1ca`.
- Final SSM status: `Success`, `ResponseCode=0`.
- Execution window: `2026-06-15T10:07:44.318Z` to
  `2026-06-15T11:22:57.318Z`, elapsed `PT1H15M13.844S`.
- Load/build result: 990000 corpus rows, 10000 queries, corpus copy 320.23s,
  encode 421.44s, query copy 3.37s, index build 2680.67s, total 3553.77s.
- Routing/fanout evidence: `load/inspect.log` and `bench/storage.log` record
  `task107_phase1_rabitq_1m_l2_idx` with `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
  `storage_format=rabitq`.
- Key recall results:
  - k10 nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 / 0.9690.
  - k100 nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 / 0.9375.
- Key latency results:
  - k10 c1 mean nprobe 8/16/24/32: 203.3 / 361.9 / 521.1 / 658.0 ms.
  - k10 c4 mean nprobe 8/16/24/32: 217.8 / 387.4 / 556.3 / 703.4 ms.
  - k10 c8 mean nprobe 8/16/24/32: 232.6 / 411.6 / 584.0 / 743.1 ms.
  - k1 c32 nprobe 32 mean: 1527.0 ms.
- Storage result: total 15.4 GiB; `ec_spire` index 168.0 KiB, 0.2 B/row with
  `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
  `storage_format=rabitq`.
- Cleanup result: `load/cleanup-drop.log` dropped the index, query table, and
  corpus table; `load/residue-after-cleanup.log` is empty.
- AWS state after cell: all three Task 107 instances remained running with
  `AutoStop=2026-06-17T05:30:31Z`, recorded in
  `aws-state/describe-after-cell.json`.
