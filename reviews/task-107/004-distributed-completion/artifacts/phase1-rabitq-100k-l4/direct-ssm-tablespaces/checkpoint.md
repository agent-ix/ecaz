# Cell Checkpoint: phase1-rabitq-100k-l4

Status: prepared; not started.

## Intent

- Checklist cell: `phase1-rabitq-100k-l4`.
- Phase: 1, single-node multi-disk / multi-store.
- Scale: 100k representative corpus.
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 4.
- Store tablespaces:
  `ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4`.
- Prefix: `task107_phase1_rabitq_100k_l4`.
- Index: `task107_phase1_rabitq_100k_l4_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l4/direct-ssm-tablespaces/`.

## Execution Policy

Run this cell to completion or command failure. On failure, package the exact
failure and proceed according to the checklist.

## Planned Work

1. Use coordinator-local SSM to load the existing 100k representative corpus
   from S3.
2. Drop only `task107_phase1_rabitq_100k_l4%` residue before loading.
3. Build only `task107_phase1_rabitq_100k_l4_idx` with
   `local_store_count=4` and explicit store tablespaces.
4. Run the packet-local `ecaz bench suite` config for this prefix.
5. Capture storage evidence.
6. Clean up only `task107_phase1_rabitq_100k_l4%` objects.

No Task 106 single-store evidence, comparator baselines, or other index cells
are rerun as part of this cell.
