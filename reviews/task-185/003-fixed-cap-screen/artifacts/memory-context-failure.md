# Basin diagnostic memory-context failure

Date: 2026-07-23 (America/Los_Angeles)

The first release-profile 100k screen at runner SHA
`56b6b674aeb55f480f612af97310d9c8a066ab60` was stopped before measurement
because its frequency-control basin diagnostic was not benchmark-valid.

The diagnostic evaluated all 200 queries in one SQL statement using a LATERAL
call to `ec_distann_physical_seed_basin_benchmark`. The coordinator backend
grew monotonically while that statement remained active:

- PID `1669072` reached 43,002,796 KiB RSS at 37:03 elapsed;
- it then reached 47,805,824 KiB RSS and 72.6% of the 64,224 MiB host;
- host swap use reached 4,770 MiB and available memory fell to 13,116 MiB;
- the backend alternated between CPU work and `folio_wait_bit_common`, so
  latency from that run would have included severe swapping; and
- after the suite client exited, SIGINT and SIGTERM could not cancel the
  backend while it was blocked in swap I/O. The orphaned backend was
  force-killed, after which host available memory recovered to 55,630 MiB.

Root cause: pgrx allocations made by the benchmark-only function remained in
PostgreSQL's statement memory context across the 200 LATERAL evaluations.
The query therefore accumulated all query-conditioned basin work until the
statement ended.

Fix: commit `df89b57264adf49903a9f407c76053510e4cb30b` executes the same
deterministically ordered 200-query population as 200 one-query statements and
aggregates the four returned fields in the CLI. Each statement reset bounds
live backend memory to one query. The failed attempt produced no recall,
latency, or storage result and is not used in the Task 185 decision.

The first corrected rerun then exposed the identical retention pattern in the
older Task 181 seed-coverage diagnostic, which still used one 200-row LATERAL
statement. It was stopped before swap pressure after the coordinator grew
from 5,904,392 KiB to 20,528,128 KiB RSS in under three minutes. The same
repair was therefore applied to coverage: 200 deterministically ordered
one-query statements, with the original rates, score-gap percentiles, and
region histogram aggregated in the CLI. This second attempt also produced no
measurement result and is excluded from the decision.
