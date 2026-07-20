# Packet 046 system-column rejection isolation

This is an exact historical A/B of packet 046. The before extension is the
checkpoint parent `2af20eb0785e18f9f97504c8cb52740d2de85c28`; the after
extension is `754eb7b911bf5aa5e2c6e7d4adb8213d03ff5b06`. Both arms use the
same release `ecaz bench suite` runner, staged real corpora, PG18 host,
three-owner topology, degree 32, head cap 4096, BW4/H100, top-k 10, 20
queries, 200 recall trials, 10 warmups, and 30 warm-cache latency iterations.

| Scale | Recall before -> after | Recall-workload mean ms before -> after | Physical p95 ms before -> after | Generation bytes before -> after | Control bytes before -> after |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10k | `1.0000 -> 1.0000` | `546.65 -> 546.54` | `51.9 -> 51.2` | `242,745,344 -> 242,761,728` | `24,576 -> 24,576` |
| 50k | `0.9800 -> 0.9800` | `490.90 -> 570.25` | `67.6 -> 68.8` | `1,242,734,592 -> 1,242,734,592` | `24,576 -> 24,576` |
| 100k | `0.9500 -> 0.9500` | `825.02 -> 921.95` | `66.9 -> 66.2` | `2,496,659,456 -> 2,496,659,456` | `24,576 -> 24,576` |

Measured disposition: measured recall is identical at every scale.
Generation storage is equivalent at 50k/100k and differs by one 16 KiB page
at 10k; control-index storage is equivalent throughout. Warm physical p95 is
mixed but close in this single run (`-1.3%`, `+1.8%`, `-1.0%`), while the
broader 200-trial recall-workload mean is mixed (`-0.02%`, `+16.2%`,
`+11.7%`). This packet therefore makes **no speedup or latency-neutrality
claim**.

The checkpoint adds planner-time rejection for relation-local system-column
Vars and a plan-construction backstop. The suite intentionally uses supported
user-column queries: it measures whether the new planner walk adds unintended
hot-path latency or changes recall/storage, while packet 046's PG18 regression
tests remain the correctness evidence for the new rejection branch. The exact
matrix closes the missing-isolation finding without attributing the noisier
recall-workload timing to the small planner-only checkpoint.
