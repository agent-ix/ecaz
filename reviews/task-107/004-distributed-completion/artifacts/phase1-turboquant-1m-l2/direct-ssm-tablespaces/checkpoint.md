# Cell Checkpoint: phase1-turboquant-1m-l2

Status: completed.

## Intent

- Checklist cell: `phase1-turboquant-1m-l2`.
- Phase: 1, single node with 2 disks.
- Scale: 1m representative corpus.
- Storage format: TurboQuant.
- Bits: 4.
- Store count: 2.
- Store tablespaces: `ecaz_spire_store_1,ecaz_spire_store_2`.
- Prefix: `task107_phase1_turboquant_1m_l2`.
- Index: `task107_phase1_turboquant_1m_l2_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-turboquant-1m-l2/direct-ssm-tablespaces/`.

## Result

- SSM command id: `74873503-5793-43b0-b65e-6ff92c8bc08d`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT1H15M19.4S`.
- Load/build result: 990000 corpus rows, 10000 queries, `bits=4`,
  `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
  `storage_format=turboquant`; copy 320.20s, encode 449.26s, query copy
  5.72s, index build 2678.40s, total 3583.10s.
- Recall k10 nprobe 8/16/24/32/64: 0.8110 / 0.8820 / 0.9060 / 0.9340 /
  0.9690.
- Recall k100 nprobe 8/16/24/32/64: 0.7626 / 0.8425 / 0.8763 / 0.8988 /
  0.9375.
- Latency k10 c1 mean nprobe 8/16/24/32: 188.1 / 343.6 / 499.1 / 620.1 ms.
- Latency k10 c4 mean nprobe 8/16/24/32: 214.6 / 380.1 / 542.3 / 686.4 ms.
- Latency k10 c8 mean nprobe 8/16/24/32: 233.2 / 410.0 / 583.9 / 732.6 ms.
- Latency k1 c32 nprobe32 mean: 1525.1 ms.
- Storage result: total 15.4 GiB; `ec_spire` index 168.0 KiB with
  `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
  `storage_format=turboquant`.
- Cleanup result: dropped index, queries table, and corpus table; no remaining
  `task107_phase1_turboquant_1m_l2%` relations were printed in
  `load/residue-after-cleanup.log`.

## Execution Policy

Ran this cell to completion using `ecaz bench suite`.
Do not run any single-node/single-disk, 4-disk, or comparator rows as part of
this cell.

## Planned Work

1. Use coordinator-local SSM to load the existing 1m representative corpus
   from S3.
2. Drop only `task107_phase1_turboquant_1m_l2%` residue before loading.
3. Build only `task107_phase1_turboquant_1m_l2_idx` with
   `local_store_count=2`, `storage_format=turboquant`, and explicit store
   tablespaces.
4. Run the packet-local `ecaz bench suite` config for this prefix.
5. Capture storage evidence.
6. Clean up only `task107_phase1_turboquant_1m_l2%` objects.
