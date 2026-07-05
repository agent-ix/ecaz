# Task 87 Packet 014: Phase 6 Real100k Matrix

## Summary

This packet asks for review of the Task 87 Phase 6 real100k measurement slice.
It follows packet 012's approved suite shape and packet 013's truth-cache
correction for reused surfaces. It does not claim Phase 6 closeout yet.

Current head for this packet:

- `85a238c38dcc30612af20edcf916f033632f8e72` - `Add Task 87 real50k matrix packet`

The packet includes packet-local artifacts for:

- HNSW off/on recall, latency, and storage on
  `current_intel_real100k_hnsw`.
- IVF off/on recall, latency, and storage on
  `task28_ivf_tq100k_n64w25`.
- SPIRE off/on pipeline metrics and storage on
  `task74_intel_spire_highrecall_tg128_b0`.
- A packet-local suite copy so the 100k logs and truth caches are written
  directly into this packet.

## Real100k Results

All cells used PG18 on `/home/peter/.pgrx:28818`.

| AM | off recall | on recall | off latency | on latency | storage |
| --- | ---: | ---: | ---: | ---: | ---: |
| HNSW | 0.8980, mean q-time 61.99 ms | 0.8980, mean q-time 36.71 ms | p50 35.6 ms, p95 43.8 ms, p99 51.6 ms | p50 35.5 ms, p95 42.9 ms, p99 50.1 ms | total 1.7 GiB, indexes 132.4 MiB |
| IVF | 1.0000, mean q-time 1093.79 ms | 1.0000, mean q-time 983.10 ms | p50 1064.2 ms, p95 1114.9 ms, p99 1131.1 ms | p50 960.5 ms, p95 1018.0 ms, p99 1048.6 ms | total 1.6 GiB, indexes 89.5 MiB |
| SPIRE | 0.9100, p50 414.768 ms, p95 471.651 ms, p99 495.905 ms | 0.9100, p50 273.031 ms, p95 298.031 ms, p99 308.541 ms | pipeline query metrics | pipeline query metrics | total 1.6 GiB, indexes 81.8 MiB |

## Notes

- Recall is unchanged across off/on for all three AMs in this slice.
- HNSW recall mean q-time improves materially, but latency p50 is effectively
  flat on this 100k surface.
- IVF and SPIRE both show material p50/p95/p99 improvements on this 100k slice.
- SPIRE endpoint identity reports local tuple transport ready and remote
  serving status `requires_rabitq_storage_format`, expected for this local
  TurboQuant surface and not a blocker for the local pipeline cell.

## Review Focus

- Confirm the real100k evidence is acceptable as the third Phase 6 matrix
  checkpoint.
- Confirm the packet-local suite copy is acceptable for keeping the 100k
  artifacts self-contained while preserving the approved suite shape.
- Confirm HNSW's flat 100k latency result and IVF/SPIRE improvements are
  documented clearly enough for aggregate closeout handling.
