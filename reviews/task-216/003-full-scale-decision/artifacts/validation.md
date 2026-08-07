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
scale. Fault drills were explicitly skipped by the checked-in config
(`skip_fault_drills: true`); no outage-drill coverage is claimed. No NFR-021
conformance claim is made.

The stage counters put the owner SQL work at 40.376 ms summed over owners for
control and 118.422 ms for MAT-15. Coordinator decode was 0.076 ms and 0.096
ms per scan respectively. Returned payload bytes were 576,576 and 576,945.
Thus MAT-15's addressable ceiling on this profile is 0.19% of the control
scan, and there is no wire-byte reduction. The implementation regression is a
secondary diagnostic, not the family-closing rationale.

Carry-ins for MAT-21 or any successor candidate are: (1) build one generation
and swap only the extension binary, or pin the drifting generation input,
before asserting ordered-result identity; and (2) require a maximum-
addressable-win calculation in candidate screening before advancing a
stage-local hypothesis.
