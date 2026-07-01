# Task 131 Packet 022: Real-Scale Threshold Boundability

## Summary

This packet runs the reviewer-requested real-scale boundability profile through `ecaz bench suite` on the staged real corpora. The completed 10k and 50k cells all say the same thing: current scan-time instrumentation has no sound per-list/per-row upper bound available to apply the candidate-derived global threshold.

The practical conclusion is that a gated Phase 3 early-stop implementation is not currently implementable from the existing scan metadata. Building it now would only wire a threshold through code that has nothing sound to compare against. The next decision should be a Phase 4 metadata/design decision for usable upper bounds, or shelving the early-stop branch until such metadata exists.

## Evidence

Packet artifacts:

- `artifacts/manifest.md`
- `artifacts/task131-phase3-real-scale-boundability-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/suite-run.log`
- `artifacts/threshold-boundability-summary.md`
- per-cell `bench-suite/results.jsonl` files for `10k-n128-b4`, `10k-n1024-b2`, `50k-n128-b4`, and `50k-n1024-b2`

Completed cells:

| cell | recall@10 | p50 | p95 | bound result |
| --- | ---: | ---: | ---: | --- |
| `10k-n128-b4` | `1.0000` | `613.826 ms` | `673.385 ms` | all nodes `sound_bound_available_sum=0`, block/row skip sums `0` |
| `10k-n1024-b2` | `0.9750` | `528.689 ms` | `672.335 ms` | all nodes `sound_bound_available_sum=0`, block/row skip sums `0` |
| `50k-n128-b4` | `1.0000` | `2703.202 ms` | `3371.223 ms` | all nodes `sound_bound_available_sum=0`, block/row skip sums `0` |
| `50k-n1024-b2` | `1.0000` | `644.494 ms` | `833.718 ms` | all nodes `sound_bound_available_sum=0`, block/row skip sums `0` |

The suite did not complete the full requested six-cell matrix. It failed during `100k-n128-b4` remote node 4 setup because the workspace filesystem hit 100% usage:

```text
ERROR: could not extend file "base/5/17625": No space left on device
```

The failed 100k logs are kept in the packet. `100k-n1024-b2` was not reached.

## Review Ask

Please review the decision point, not the already-retired heap path:

- Are the four completed cells enough to accept that current metadata exposes no usable sound bound for Phase 3 early stop?
- Should Task 131 be redirected to a narrow Phase 4 bound-metadata design packet, rather than implementing threshold plumbing that cannot skip?

I did not request closure of Task 131 from this packet.
