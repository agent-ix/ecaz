---
task: 172
packet: 011-final-gate
role: coder
status: review-requested
head: c14c87aab351112963f8257bcd8b416943584a3c
date: 2026-08-08
---

# Review request: final physical 10k/50k/100k benchmark gate

## Requested decision

Please review commit `c14c87aab351112963f8257bcd8b416943584a3c` and the
decision-bearing suite evidence in this packet. This is the final Task 172
matrix after prerequisites 204, 205, 206, and 208 were review-closed.

The matrix ran through `ecaz bench suite` in release mode on the real
three-instance physically hash-sharded path at 10k, 50k, and 100k, with a
single-instance control at each scale. It also includes a 10k full-metrics
diagnostic arm. All four suite steps succeeded, and the suite-level NFR-021
cross-scale check passed.

## Decision summary

- NFR-021 is conforming across all three decision scales: zero non-owned
  records, zero orphan vectors, zero coordinator-resident unsharded bytes, and
  zero missing owned records.
- Remote-path engagement passed at every scale: two remote owners and two
  materialization probes.
- Physical recall is `1.0000`, `0.9750`, and `0.9550` at 10k, 50k, and 100k;
  the single control is `1.0000`, `0.9800`, and `0.9500`. The confidence
  intervals overlap at every scale, so this run shows no recall collapse.
- Physical cluster index-space amplification is `1.235600`, `1.332693`, and
  `1.351147` at 10k, 50k, and 100k.
- At concurrency 1, physical throughput is `56.185`, `50.630`, and `49.379`
  QPS. At concurrency 16 it is `14.412`, `19.031`, and `18.794` QPS. The
  local physical fixture is materially slower than its single-instance
  control, so this packet does not claim a distributed performance win.
- The 10k full-metrics diagnostic was effectively the same as benchmark mode
  for the physical arm (`17.80 ms` versus `17.60 ms` mean latency in this
  run); it is diagnostic evidence, not a replacement for the gate matrix.

The recommended disposition is to accept Task 172 as a completed measurement
gate: the physical distributed path is correct, engaged, and recall-neutral in
this matrix, while performance promotion remains unsupported by these local
numbers. Task 219 may proceed as the separate Pareto-recall decision.

## Validation

See `artifacts/manifest.md` for commands, provenance, and artifact inventory;
`artifacts/final-verdict.md` for the criterion-by-criterion disposition; and
`artifacts/results.jsonl` for the normalized suite output.

- Suite audit: pass, four configured steps.
- Full suite run: pass, all four steps succeeded.
- Cross-scale NFR-021 validator: pass for `100k,10k,50k`.
- Release preflight: unanimous, extension SHA
  `22ed70bb9d5a39685f0c06db40a4491489516da6`.
- No corpus or cluster data is committed. Cluster run directories are
  external to the repository and are removed after packet capture.

This request remains open for outside review.
