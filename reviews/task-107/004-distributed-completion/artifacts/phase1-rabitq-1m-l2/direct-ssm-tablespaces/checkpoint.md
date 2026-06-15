# Cell Checkpoint: phase1-rabitq-1m-l2

Status: prepared; not started.

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

The SSM command is sent with a high AWS service timeout and
`AWS-RunShellScript` `executionTimeout=172800`; there is no benchmark time cap
in the payload.
