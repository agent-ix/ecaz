# Task 219 measured release frontier

The current release frontier is the two accepted Task 215 arms. Each scale
has a shipped BW4/H100 control and a BW64/H8 candidate; no intermediate point
is needed to decide whether the measured candidate is an acceptable default
under the current release contract.

| Scale | Control recall | Candidate recall | Control mean ms | Candidate mean ms | Control storage bytes | Candidate storage bytes | Candidate latency delta |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 0.9990 | 0.9995 | 18.80 | 22.60 | 242,753,536 | 242,745,344 | +20.2% |
| 50k | 0.9545 | 0.9900 | 20.80 | 29.00 | 1,242,750,976 | 1,242,742,784 | +39.4% |
| 100k | 0.9280 | 0.9815 | 21.40 | 31.60 | 2,496,651,264 | 2,496,659,456 | +47.7% |

Source for every row: `reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/results.jsonl`.
The compact paired values and confidence intervals are also recorded in the
source packet's `artifacts/decision.md`. Task 206's 194--231 ms values are
excluded from this frontier because that task used `top_k=200`/L200 rather
than this release lane's `top_k=10`/effective L64.

The candidate's recall deltas are +0.0005, +0.0355, and +0.0535. Storage is
effectively unchanged, so the observed trade is recall for latency rather than
a storage improvement.
