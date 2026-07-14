# Task 179 complete-finding benchmark comparison

All arms use PG18, the same staged real-corpus fixtures, three physical
owners, graph degree 32, head cap 4096, top-k 10, 20 queries, 200 recall
trials, 10 latency warmups, and 30 measured warm-cache latency iterations.
The baseline extension is `0b2d4fbabedb4caa59535b875b81359b4dd6f91c`.
The candidate extension was installed from
`45f9f0f980d9548083aa965659df4de7089b6e18`; its owner fixed-cost
implementation is `4587c0d09` and its subsequent lifecycle-only refactor is
`0043c3e74`. Both sides were driven by the same release CLI runner at
`45f9f0f98`.

## Fixed-cost A/B: baseline versus candidate default (BW4/H100)

| Scale | Physical recall | Recall-workload mean ms | Change | Physical p95 ms | Change | Generation bytes (before -> after) | Control bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 -> 1.0000 | 604.22 -> 53.21 | -91.2% | 53.6 -> 45.9 | -14.4% | 242,761,728 -> 242,745,344 | 24,576 -> 24,576 |
| 50k | 0.9800 -> 0.9800 | 612.36 -> 65.41 | -89.3% | 67.8 -> 59.0 | -13.0% | 1,242,750,976 -> 1,242,734,592 | 24,576 -> 24,576 |
| 100k | 0.9500 -> 0.9500 | 1,075.80 -> 60.45 | -94.4% | 68.5 -> 58.3 | -14.9% | 2,496,626,688 -> 2,496,659,456 | 24,576 -> 24,576 |

Recall is unchanged at every scale. The candidate removes the repeated owner
query fixed cost, producing an 89-94% reduction in the 200-trial recall
workload mean and a 13-15% reduction in warm p95. Storage is effectively
unchanged, and the explicit aggregate control index is 24,576 bytes at every
scale. Physical build time moved from 66,867/416,605/886,325 ms to
76,241/427,264/913,026 ms; this packet claims no build-time win.

## Fixed-product search shape: candidate BW4/H100 versus BW16/H25

| Scale | Recall (default -> BW16/H25) | Physical p95 ms (default -> BW16/H25) | P95 change | Disposition |
| --- | ---: | ---: | ---: | --- |
| 10k | 1.0000 -> 1.0000 | 45.9 -> 64.5 | +40.5% | reject as default |
| 50k | 0.9800 -> 0.9950 | 59.0 -> 113.0 | +91.5% | explicit recall/latency tradeoff |
| 100k | 0.9500 -> 0.9600 | 58.3 -> 92.1 | +58.0% | explicit recall/latency tradeoff |

BW16/H25 keeps the nominal `beam_width * hop_rounds` product at 400, but it
is materially slower at every scale. It offers a modest recall increase at
50k and 100k, so it remains a selectable quality/latency tradeoff rather than
the default.

## Coordinator outside the roster

| Scale | Remote owners engaged | Topology gate | Physical recall | Physical p95 ms | Generation bytes | Control bytes |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| 10k | 3 | pass | 1.0000 | 45.3 | 242,778,112 | 24,576 |
| 50k | 3 | pass | 0.9800 | 65.3 | 1,242,726,400 | 24,576 |
| 100k | 3 | pass | 0.9500 | 58.7 | 2,496,659,456 | 24,576 |

All three physical owners were remote from the coordinator and materially
engaged at every scale. Every ready and published topology row reported zero
non-owned rows and zero orphans.

## Evidence dispositions

- The BW16/H25 100k first attempt reached Published topology and then failed
  while building the single-index control because the filesystem filled. The
  task-local PostgreSQL run directories were removed, the successful 10k/50k
  records were reused with `--resume-from`, and the 100k step reran to a clean
  3/3-success manifest. Both attempt manifests and runner logs are retained.
- No 1m staged fixture exists under `data/staged-current`; that directory has
  only 10k, 50k, and 100k corpus/query/manifest triples. The repository's
  mandatory minimum is therefore fully covered, but this packet makes no 1m
  claim.
- These runs recapture `control_index_bytes` at all required scales. They do
  not infer a heap-versus-TOAST split that the runner does not emit; packet
  032 records that historical probe gap explicitly.
