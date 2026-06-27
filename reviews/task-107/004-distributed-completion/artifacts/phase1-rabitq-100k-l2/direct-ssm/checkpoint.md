# Cell Checkpoint: phase1-rabitq-100k-l2

Status: prepared; not started.

## Intent

- Checklist cell: `phase1-rabitq-100k-l2`.
- Phase: 1, single-node multi-disk / multi-store.
- Scale: 100k representative corpus.
- Storage format: RaBitQ.
- Bits: 4.
- Store count: 2.
- Prefix: `task107_phase1_rabitq_100k_l2`.
- Index: `task107_phase1_rabitq_100k_l2_idx`.
- Artifact directory:
  `reviews/task-107/004-distributed-completion/artifacts/phase1-rabitq-100k-l2/direct-ssm/`.

## Execution Policy

Run the cell to completion or command failure. On failure, package the exact
failure and proceed according to the checklist.

## Planned Work

1. Use coordinator-local SSM to load the existing 100k representative corpus
   from S3.
2. Drop only `task107_phase1_rabitq_100k_l2%` residue before loading.
3. Build only `task107_phase1_rabitq_100k_l2_idx` with
   `local_store_count=2`.
4. Run the packet-local `ecaz bench suite` config for this prefix.
5. Capture storage evidence.
6. Clean up only `task107_phase1_rabitq_100k_l2%` objects.

No Task 106 single-store evidence, comparator baselines, or other index cells
are rerun as part of this cell.
