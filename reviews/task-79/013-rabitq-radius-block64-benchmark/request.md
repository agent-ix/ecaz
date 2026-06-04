# Review Request: RaBitQ Radius Block64 Benchmark

Code commit measured: `2a7c7a089ffe5e45344c32001c9139c0e6cd0c55`

This packet benchmarks the radius-adjusted RaBitQ selector from packet 012 on
the primary/default Task 79 RaBitQ lane. It is a negative benchmark packet.

## Result

Radius-adjusted block64 pruning reduces candidates and p50 latency, but recall
is still far below the Task 79 gate and is worse than the packet 011 mean-only
selector at comparable budgets.

| Step | Candidate sum | p50 latency | p95 latency | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| block64 radius prune0 | 15,506,227 | 62.127 ms | 69.806 ms | 0.9975 |
| block64 radius prune4 | 4,681,394 | 36.993 ms | 40.858 ms | 0.6640 |
| block64 radius prune6 | 6,883,846 | 42.145 ms | 48.145 ms | 0.7940 |
| block64 radius prune8 | 8,896,977 | 47.395 ms | 54.075 ms | 0.8835 |

For direct comparison, packet 011 mean-only block64/prune4 was
`4,547,347` candidates, `37.292 ms` p50, and `0.7790` recall@10. Radius is not
an improvement.

## Interpretation

The candidate-surface reduction path is real, but single-summary block selection
is still the wrong abstraction. Adding a residual radius over-selects broad
blocks without recovering the high-recall rows that matter.

This should not be accepted as a Task 79 latency improvement. It is useful
evidence because it rules out the simplest upper-bound variant and narrows the
next attempt toward multi-representative or membership-aware block selection.

TurboQuant was not benchmarked here because the primary RaBitQ lane failed first.

## Validation And Artifacts

See `artifacts/manifest.md`.

- `ecaz bench suite audit`: pass
- `ecaz bench suite run --dry-run`: pass
- `cargo build -p ecaz-cli`: pass
- `ecaz dev install ecaz-pg-test --pg 18`: pass, backend SHA256 `a85aabc5a008ff2d0f2649fe8b7f992ef20ca714199db175a2b6b285dcb2f60a`
- final `ecaz bench suite run`: 6 steps succeeded, 0 failed
- `ecaz bench suite status`: completed 6, failed 0, stale 0
- `ecaz bench suite report`: produced `artifacts/report-results.jsonl`

## Next Work

The next direct candidate-reduction slice should stop relying on one encoded
centroid per block. Reasonable next experiments are:

- store multiple representatives per block and select blocks by the best
  representative score
- store a compact per-block candidate sketch based on training queries or row
  top-k membership
- use summary scoring only as a first pass and add a tiny row-sample probe per
  block before committing to row ranges
