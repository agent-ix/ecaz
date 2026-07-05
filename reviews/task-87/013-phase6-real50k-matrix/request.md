# Task 87 Packet 013: Phase 6 Real50k Matrix

## Summary

This packet asks for review of the Task 87 Phase 6 real50k measurement slice.
It follows packet 012's approved suite shape and does not claim Phase 6
closeout yet.

Current head for this packet:

- `ed5338d07c8d3b9e9071c6ab2281119373b846b54` - `Add Task 87 real10k matrix slice artifacts`

The packet includes packet-local artifacts for:

- HNSW off/on recall, latency, and storage on
  `task67_current_shape_real50k_hnsw`.
- IVF off/on recall, latency, and storage on
  `task67_local_50k_ivfrabitq`.
- SPIRE off/on pipeline metrics and storage on
  `task87_phase6_real50k_spire`.
- AM-specific truth caches where reused surfaces have different query tables.

## Suite Adjustment

The checked-in suite config in packet 012 was updated before this run so the
remaining 50k/100k recall cells use 100-query truth caches and the reused IVF
surfaces use AM-specific truth-cache files. The 50k query table hashes showed:

- HNSW and SPIRE real50k query tables match.
- IVF real50k has a distinct query table and needs
  `truth-real50k-ivf-k10.json`.

SPIRE pipeline requires its truth cache to exist before `spire-pipeline` runs,
so this packet includes `truth-real50k-spire-generate.log`.

## Real50k Results

All cells used PG18 on `/home/peter/.pgrx:28818`.

| AM | off recall | on recall | off latency | on latency | storage |
| --- | ---: | ---: | ---: | ---: | ---: |
| HNSW | 0.9180, mean q-time 43.66 ms | 0.9180, mean q-time 32.47 ms | p50 32.4 ms, p95 42.1 ms, p99 58.5 ms | p50 31.3 ms, p95 37.9 ms, p99 41.9 ms | total 860.0 MiB, indexes 66.2 MiB |
| IVF | 0.9300, mean q-time 266.77 ms | 0.9300, mean q-time 264.18 ms | p50 264.0 ms, p95 289.7 ms, p99 311.6 ms | p50 264.3 ms, p95 292.9 ms, p99 308.5 ms | total 840.9 MiB, indexes 47.1 MiB |
| SPIRE | 0.9690, p50 224.610 ms, p95 255.674 ms, p99 266.182 ms | 0.9690, p50 160.449 ms, p95 180.921 ms, p99 186.580 ms | pipeline query metrics | pipeline query metrics | total 834.3 MiB, indexes 40.5 MiB |

## Notes

- Recall is unchanged across off/on for all three AMs in this slice.
- HNSW and SPIRE show latency improvements with candidate-batch scoring on.
- IVF RaBitQ is effectively flat/slightly worse in p50/p95 on this reused 50k
  surface; this should be carried into the aggregate closeout rather than
  hidden.
- SPIRE endpoint identity reports local tuple transport ready and remote
  serving status `requires_rabitq_storage_format`, expected for this local
  TurboQuant surface and not a blocker for the local pipeline cell.

## Review Focus

- Confirm the real50k evidence is acceptable as the second Phase 6 matrix
  checkpoint.
- Confirm the AM-specific truth-cache correction is acceptable for reused
  surfaces with distinct query tables.
- Confirm the IVF flat/slightly-worse 50k RaBitQ result is documented clearly
  enough for aggregate closeout handling.
