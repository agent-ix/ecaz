# Task 94 Packet 027: Latency / Width Rerun Follow-up

Code checkpoint: `a808ee5c0c6ecd7a3fac9d8fbcf38bfd77dfa3cf` (`Address Task 101 width-cascade review cleanup`)

This packet responds to Task 94 packet 026 feedback findings 1 and 2.

## What Changed

No Task 94 F8 kernel code changed in this checkpoint. The code commit is the Task 101 cleanup for dead helpers and scalar-host partial-tail behavior.

## Evidence Updates

Artifacts: `reviews/task-94/027-latency-width-rerun/artifacts/`

- Rebuilt and installed PG18 pg_test extension through `ecaz dev install ecaz-pg-test --pg 18`.
- Captured installed backend SHA: `d5d0a6009e2b9fe9158a40ff88ded13114e2c2403e8778f91098eb75d5fbc3ba`.
- Diagnosed the width-field failure as a stale database SQL catalog:
  - installed SQL had widened `ec_block_kernel_scoring_snapshot()`;
  - local `postgres` still had the old function signature;
  - refreshed only that function signature, without dropping fixture data.
- Reran a two-step suite with a head-built CLI:
  - one IVF grouped-PQ latency cell;
  - one DiskANN grouped-PQ latency cell.

## Key Results

- Suite status: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Width buckets are now present in raw CLI lines and parsed suite output.
- Both rerun cells have `scalar_candidates=0`.

Fresh-cache latency observations:

| Cell | p50 |
| --- | ---: |
| IVF 10k batch-on `nprobe=32` | 2.70 ms |
| IVF 10k batch-on `nprobe=64` | 4.10 ms |
| DiskANN 50k grouped-PQ `list_size=64` | 15.3 ms |
| DiskANN 50k grouped-PQ `list_size=128` | 15.5 ms |

Width histogram examples:

| Cell | width_lt8 | width_8_15 | width_16_31 | width_ge32 |
| --- | ---: | ---: | ---: | ---: |
| IVF 10k `nprobe=32` | 15 | 20 | 40 | 9605 |
| IVF 10k `nprobe=64` | 0 | 0 | 500 | 19500 |
| DiskANN 50k `list_size=64` | 970 | 2531 | 4873 | 201 |
| DiskANN 50k `list_size=128` | 3003 | 5295 | 7515 | 206 |

## Latency Diagnosis

The stale CLI/catalog issue is fixed, but it was not the full explanation for packet 026's latency shift. A diagnostic full-latency rerun using the original matrix cache-state names reproduced the slow packet-026 profile and was stopped before completing the long 100k batch-on cell. That diagnostic output is included but explicitly not used as closeout evidence.

Please treat this packet as closing the missing width-histogram/provenance finding and narrowing the latency issue. AC5 still should not be closed from packet 026 or from the interrupted full diagnostic matrix.
