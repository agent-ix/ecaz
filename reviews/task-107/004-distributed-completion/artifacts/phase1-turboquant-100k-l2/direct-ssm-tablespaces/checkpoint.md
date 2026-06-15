# Cell Checkpoint: phase1-turboquant-100k-l2

Status: completed.

## Intent

- Checklist cell: `phase1-turboquant-100k-l2`.
- Phase: 1, single node with 2 disks.
- Scale: 100k representative corpus.
- Storage format: TurboQuant.
- Bits: 4.
- Store count: 2.
- Store tablespaces: `ecaz_spire_store_1,ecaz_spire_store_2`.
- Prefix: `task107_phase1_turboquant_100k_l2`.
- Index: `task107_phase1_turboquant_100k_l2_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-turboquant-100k-l2/direct-ssm-tablespaces/`.

## Result

- SSM command id: `6f98340e-4524-41fb-8938-54c4c6c72fc7`.
- SSM result: `Status=Success`, `ResponseCode=0`, elapsed `PT36M54.883S`.
- Load/build result: 100000 corpus rows, 1000 queries, `bits=4`,
  `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`,
  `storage_format=turboquant`; copy 32.46s, encode 23.94s, index build
  90.02s, total 159.82s.
- Recall k10 nprobe 8/16/24/32/64: 0.7939 / 0.8703 / 0.9041 / 0.9268 /
  0.9661.
- Recall k100 nprobe 8/16/24/32/64: 0.6862 / 0.7899 / 0.8362 / 0.8687 /
  0.9336.
- Latency k10 c1 mean nprobe 8/16/24/32: 47.6 / 82.8 / 117.4 / 159.4 ms.
- Latency k10 c4 mean nprobe 8/16/24/32: 49.9 / 83.4 / 122.5 / 156.2 ms.
- Latency k10 c8 mean nprobe 8/16/24/32: 49.0 / 84.6 / 121.6 / 159.8 ms.
- Latency k1 c32 nprobe32 mean: 327.1 ms.
- Storage result: total 1.6 GiB; `ec_spire` index 64.0 KiB with
  `local_store_count=2`,
  `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`, and
  `storage_format=turboquant`.
- Cleanup result: dropped index, queries table, and corpus table; no remaining
  `task107_phase1_turboquant_100k_l2%` relations were printed in
  `load/residue-after-cleanup.log`.

## Execution Policy

Ran this cell to completion using `ecaz bench suite`.
Do not run any single-node/single-disk, 4-disk, or comparator rows as part of
this cell.

## Planned Work

1. Use coordinator-local SSM to load the existing 100k representative corpus
   from S3.
2. Drop only `task107_phase1_turboquant_100k_l2%` residue before loading.
3. Build only `task107_phase1_turboquant_100k_l2_idx` with
   `local_store_count=2`, `storage_format=turboquant`, and explicit store
   tablespaces.
4. Run the packet-local `ecaz bench suite` config for this prefix.
5. Capture storage evidence.
6. Clean up only `task107_phase1_turboquant_100k_l2%` objects.
