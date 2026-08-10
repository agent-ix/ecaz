# Task 221 MAT-22 decision

## Verdict

STOP. The candidate is correctness-safe but not a useful isolated latency
optimization at the 100k gate. The preregistration says a neutral or
regressing end-to-end/custom-scan result stops the work and does not authorize
the 10k/50k/100k matrix.

## Isolated A/B result

| metric | control | candidate | delta |
| --- | ---: | ---: | ---: |
| recall | 0.9290 | 0.9290 | 0 |
| membership recall | 0.9290 | 0.9290 | 0 |
| warm mean latency | 21.60 ms | 21.90 ms | +0.30 ms (+1.39%) |
| warm p50 | 21.50 ms | 21.50 ms | 0 |
| warm p95 | 25.30 ms | 25.70 ms | +0.40 ms (+1.58%) |
| warm p99 | 25.90 ms | 26.10 ms | +0.20 ms (+0.77%) |
| custom_scan_total | 18.935517 ms | 19.164216 ms | +0.228699 ms (+1.21%) |
| owner endpoint critical | 6.099693 ms | 6.022825 ms | -0.076868 ms (-1.26%) |
| owner node lookup work | 0.310968 ms | 0 ms | -0.310968 ms |
| owner payload SQL work | 9.181797 ms | 9.332559 ms | +0.150762 ms (+1.64%) |
| physical generation bytes | 3,188,072,448 | 3,188,072,448 | 0 |

The candidate removed the targeted lookup work, but that saving was smaller
than the added/shifted work elsewhere in the request path. The end-to-end
latency and custom-scan metrics therefore fail the preregistered useful-win
gate.

## Identity and safety gates

- same-generation recall pair: `byte_identical=true`
- control/candidate prediction files: byte-identical
- materialization correctness: all 9 scenarios passed, including null payload,
  toasted projection, mixed local/remote rows, multi-window rejection, and
  post-first-batch remote failure
- storage: identical owner graph, row-tier, control, and generation totals
- topology: 3 owners, 100,000 source rows, zero non-owned records and zero
  orphan vectors
- NFR-021 scale conformance remains `decision_eligible=false` at a single
  100k scale, with no distribution gap; this does not rescue the latency STOP

Evidence is sourced from `artifacts/results.jsonl` and the suite manifest.
