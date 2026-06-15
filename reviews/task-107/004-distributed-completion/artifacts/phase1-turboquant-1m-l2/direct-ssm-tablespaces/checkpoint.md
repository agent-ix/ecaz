# Cell Checkpoint: phase1-turboquant-1m-l2

Status: prepared; not started.

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

## Execution Policy

Run this cell to completion or command failure using `ecaz bench suite`.
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
