# Cell Checkpoint: phase1-rabitq-1m-l1-control

Status: completed.

## Intent

- Checklist cell: `phase1-rabitq-1m-l1-control`.
- Phase: 1, single-node multi-disk / multi-store control.
- Scale: 1m representative corpus (`ec_real_ann_benchmarks_anchor`, 990,000
  corpus rows and 10,000 query rows in prior packet evidence).
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 1.
- Store tablespaces: `ecaz_spire_store_1`.
- Prefix: `task107_phase1_rabitq_1m_l1`.
- Index: `task107_phase1_rabitq_1m_l1_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-1m-l1-control/direct-ssm-tablespaces/`.

## Execution Policy

Run this cell to completion or command failure. Do not stop the AWS instances
after the cell unless the user explicitly asks or a concrete cleanup failure
makes that unsafe.

## Planned Work

1. Use coordinator-local SSM to download the existing 1m representative corpus
   from S3:
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_corpus.tsv`
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_queries.tsv`
   - `representative-load/representative/coordinator/ec_real_ann_benchmarks_anchor_manifest.json`
2. Drop only `task107_phase1_rabitq_1m_l1%` residue before loading.
3. Build only `task107_phase1_rabitq_1m_l1_idx` with
   `local_store_count=1` and explicit `local_store_tablespaces=ecaz_spire_store_1`.
4. Run the packet-local `ecaz bench suite` config on the coordinator node using
   the node-local 1m corpus file for exact truth generation.
5. Capture storage evidence through the suite `storage` step.
6. Clean up only `task107_phase1_rabitq_1m_l1%` objects.

The SSM command is sent with a high AWS service timeout so the benchmark is not
bounded by the AWS Run Command default. There is no benchmark time cap in the
payload.

## Result

The first full SSM attempt timed out at the `AWS-RunShellScript` document
default while building the index; see `failure-timeout.md`. Post-timeout SQL
checks showed the index had completed and was valid, so the resume command
`5b1a96bd-3efd-4253-94f2-185475af4f55` continued from the valid loaded/indexed
state.

- Final resume SSM status: `Success`, response code `0`.
- Resume execution window: `2026-06-15T09:38:47.024Z` to
  `2026-06-15T09:53:23.024Z` (`PT14M36.389S`).
- Load/index evidence: `resume/load/index-validity.log` records 990,000
  corpus rows, 10,000 query rows, and valid `ec_spire` index
  `task107_phase1_rabitq_1m_l1_idx` with
  `{local_store_count=1,local_store_tablespaces=ecaz_spire_store_1,storage_format=rabitq}`.
- Recall/latency evidence: `resume/bench/suite-results-node.jsonl` and
  `resume/load/run-suite-resume.log`.
- Storage evidence: `resume/bench/storage.log`.
- Cleanup evidence: `resume/load/cleanup-drop.log` dropped the index, query
  table, and corpus table; `resume/load/residue-after-cleanup.log` is empty.
- AWS state after the cell:
  `aws-state/describe-after-cell.json`; all three Task 107 instances remained
  running.

Key results:

- k10 recall nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 /
  0.9690.
- k100 recall nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 /
  0.9375.
- k10 c1 mean nprobe 8/16/24/32: 187.9 / 336.2 / 487.4 / 620.9 ms.
- k10 c4 mean nprobe 8/16/24/32: 211.4 / 382.6 / 541.4 / 678.5 ms.
- k10 c8 mean nprobe 8/16/24/32: 230.3 / 404.3 / 575.3 / 721.7 ms.
- k1 c32 nprobe 32 mean: 1505.3 ms.
- Storage: total 16.1 GiB; `ec_spire` index 784.8 MiB, 831.3 B/row.
