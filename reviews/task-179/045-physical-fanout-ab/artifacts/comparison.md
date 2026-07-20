# Physical owner fanout A/B comparison

This comparison isolates implementation commit `5a48c7ee9` against its exact
pre-change parent state. Both arms use release suite runner `f11ffcafc`, three
physical owners, graph degree 32, the production-default head cap 4096, 20
recall queries at top 10, 10 untimed warmups, and 50 measured latency queries
at concurrency 1.

## Physical result

| Scale | Recall baseline | Recall candidate | Mean ms baseline | Mean ms candidate | Mean delta | p95 ms baseline | p95 ms candidate | p95 delta | Physical generation bytes baseline/candidate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 1.0000 | 1.0000 | 72.40 | 42.40 | -41.4% | 91.20 | 54.90 | -39.8% | 242,745,344 / 242,745,344 |
| 50k | 0.9800 | 0.9800 | 94.60 | 59.00 | -37.6% | 122.10 | 75.10 | -38.5% | 1,242,742,784 / 1,242,742,784 |
| 100k | 0.9500 | 0.9500 | 83.50 | 50.30 | -39.8% | 119.90 | 68.80 | -42.6% | 2,496,626,688 / 2,496,626,688 |

The candidate also reduces physical p99 from 97.50/129.90/135.30 ms to
56.70/78.80/69.30 ms at 10k/50k/100k, reductions of 41.8%, 39.3%, and 48.8%.

All six physical topology gates pass. Every scale contains exactly the source
row count across three Published owners, with `non_owned=0`, `orphans=0`, and
both remote owners verified through CustomScan and frozen-row materialization.

## Same-run control

| Scale | Single mean ms baseline | Single mean ms candidate | Delta |
| --- | ---: | ---: | ---: |
| 10k | 2.47 | 2.51 | +1.6% |
| 50k | 3.36 | 3.61 | +7.4% |
| 100k | 3.20 | 3.12 | -2.5% |

The physical reductions are therefore much larger and consistently directed
than same-run host/control movement. Physical build and publication durations
remain within approximately 1.5% between arms at every scale, as expected for
a read-fanout-only change.

## Conclusion

The measured effect supports the implementation: concurrent remote-owner
expansion and materialization preserve recall, physical storage, topology, and
remote-owner engagement while reducing fully warmed physical query latency at
all required closeout scales.
