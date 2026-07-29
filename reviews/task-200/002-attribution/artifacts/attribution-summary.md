# Task 200 attribution summary

The same 100k physical generation was used for both production latency arms.
The latency backend stayed near 260–261 MB over 300 queries with
`worker_batch_size=0`:

| arm | samples | elapsed | RSS first→last | delta | slope |
| --- | ---: | ---: | ---: | ---: | ---: |
| stage counters off | 33 | 8067 ms | 260104→261028 KB | 924 KB | 114.54 KB/s |
| stage counters on | 32 | 7817 ms | 260024→261024 KB | 1000 KB | 127.93 KB/s |

The standalone benchmark-only call
`ec_distann_physical_seed_coverage_benchmark` behaves differently. In one
backend, the 200-query lateral statement reached 6.8 GB RSS in about 65 s;
the PostgreSQL memory-context dump at the captured point reported
`Grand total: 8323959872 bytes ... 8314784136 used`. The statement was canceled
before the earlier 14 GB failure point. A separate-statement run completed 14
coverage calls before cancellation and still rose from about 1.9 GB to 3.6 GB
RSS in about 32 s.

This attributes the growth to the benchmark-only seed-coverage implementation,
not `PhysicalGenerationScan::open` on the production read path. The raw
packet-local sources are in
`../001-reproduction/artifacts/run-latency-rerun/`.
