# Task 200 early RSS observation

This is a safety-stop observation from the counters-off 100k reproduction. The
suite command and exact expanded arguments are recorded in
`suite-run.log`; the run used one 100k three-owner physical generation, one
backend (`benchmark-backend-batch-size=0`), 300 timed iterations, 10 warmups,
and the 250 ms sampler. The observed query below was a pre-latency physical
recall/coverage query, before the latency child opened its sampler.

The run was stopped after the benchmark-only
`ec_distann_physical_seed_coverage_benchmark` query began to consume host
memory. This is not evidence that the ordinary production CustomScan was the
caller.
The same coordinator backend (PID 553229) was sampled read-only with `ps` and
`pg_stat_activity`:

| query elapsed | backend RSS (kB) | backend state |
| ---: | ---: | --- |
| 00:00:24 | 2,426,804 | active |
| 00:01:07 | 7,567,108 | active |
| 00:01:46 | 11,827,812 | active |
| 00:02:25 | 14,491,976 | active |

The process was then interrupted and all three exact Task 200 PostgreSQL
clusters were stopped. No latency or recall result from this incomplete run is
used as benchmark evidence. The observation establishes that growth is
present with stage counters disabled in the benchmark-only coverage statement;
it does not attribute the growth to the production read path or establish
that stage-counter reporting is the cause. A rerun with streaming sampler
output and the optional pre-latency diagnostics skipped is required to
preserve the requested interval series through a safety stop.
