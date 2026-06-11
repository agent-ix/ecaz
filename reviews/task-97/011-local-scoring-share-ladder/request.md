# Task 97 Packet 011: Local Scoring-Share Ladder

This measurement-only packet reports same-head local evidence for the corrected Task 97 QJL fixture (`dim=1024,bits=4,seed=42`) against the Task 97 scoring-share ladder.

Code checkpoint: `c07590302f2467cc2b52f84fb856acd3c612688c`

No code changed in this packet. No GitHub CI or AWS runs were used.

## Result

The current local AVX2 QJL path does not clear the Task 97 ladder:

| Surface | Parameter | Kernel off mean | Kernel on mean | End-to-end speedup |
| --- | ---: | ---: | ---: | ---: |
| IVF | `nprobe=8` | `1.19 ms` | `1.23 ms` | `0.97x` |
| IVF | `nprobe=16` | `1.50 ms` | `1.55 ms` | `0.97x` |
| SPIRE | `nprobe=8` | `9.10 ms` | `9.04 ms` | `1.01x` |
| SPIRE | `nprobe=16` | `17.5 ms` | `17.4 ms` | `1.01x` |
| HNSW | `ef_search=32` | `1.71 ms` | `1.84 ms` | `0.93x` |

Direct SPIRE scoring counters show the same issue:

- `nprobe=8`: scalar-off `22.801836 ms`; kernel-on AVX2 plus scalar tails `24.441358 ms`; `0.93x`.
- `nprobe=16`: scalar-off `45.574628 ms`; kernel-on AVX2 plus scalar tails `49.656579 ms`; `0.92x`.

IVF kernel-on emitted AVX2 direct rows, but IVF kernel-off emitted no direct block-counter rows in this local run, so this packet does not claim an IVF direct scoring-share ratio. HNSW remains scalar-tail only under the local `m=8` fixture because graph expansions stay below block width 32.

## Request

Please review the packet as evidence for the Task 97 performance ladder and stop-condition decision. The likely next implementation slice, if we continue optimizing rather than accepting a stop condition, should be reviewed separately and should target the qjl32 AVX2 block path; this packet intentionally does not hide that optimization inside measurement cleanup.

Artifacts are documented in `artifacts/manifest.md`.
