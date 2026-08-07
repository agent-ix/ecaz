# Packet 003 validation record

The two suite runs used the same checked-in config, corpus prefix, query
SHA256, topology, graph parameters, seed policy, beam, heap, hop, iteration,
and warmup settings. Release preflight was unanimous on every node.

| metric | control | MAT-15 | disposition |
|---|---:|---:|---|
| physical recall | 0.9275 | 0.9295 | small movement; not a promotion condition |
| physical mean latency (ms) | 40.60 | 86.10 | candidate loses |
| physical p95 latency (ms) | 54.30 | 113.70 | candidate loses |
| physical p99 latency (ms) | 57.20 | 127.00 | candidate loses |
| physical storage amplification | 1.351173 | 1.351160 | effectively unchanged |
| topology orphans | 0 | 0 | pass |
| remote owner probes | 2/2 | 2/2 | pass |

The physical prediction arrays have 200 query rows; byte comparison found 2
ordered-row mismatches. The single-surface arrays are byte identical. The
physical seed digests are different between fresh control and candidate
generations, so this identity result is a reproducibility stop and is not
asserted to be caused by the payload representation.

Both suite result files record NFR-021 actual `unavailable` at this diagnostic
scale. No NFR-021 conformance claim is made.
