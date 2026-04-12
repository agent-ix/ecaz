# Review Request: C1 Negative Server Timing Rejection

## Context

Packet `271` exposed a benchmark integrity hole in the warm `cached-plan` seam.
One run produced:

- `p50=11.094ms`
- `mean=3.636ms`
- `min=-799.355ms`

The percentiles were plausible, but the negative minimum and nonsense mean were
not. On this WSL2 host, the server-side `clock_timestamp()` timing seam can see
wall-clock steps, and the current launcher silently accepts those samples.

That is not acceptable for C1 because it can print a believable summary line for
an invalid run.

## Problem

`scripts/bench_sql_latency.sh` parses non-`EXPLAIN` timing modes by reading one
floating-point number per query and then summarizing them directly. It does not
reject negative values, so a backward wall-clock step can poison the mean while
still leaving p50/p95/p99 looking plausible.

## Planned work

1. Harden the Python summary path in `scripts/bench_sql_latency.sh` to reject
   negative per-query timings for server-side timing modes.
2. Surface a clear failure message so the operator reruns the cell instead of
   trusting a bogus summary.
3. Keep `EXPLAIN` mode unchanged.
4. Run the required checkpoint gate plus a shell-level smoke read for the new
   failure mode.

## Outcome

Kept.

The launcher now rejects negative per-query timings for non-`EXPLAIN` modes
before it computes the summary, so a wall-clock step can no longer print a
bogus mean that still looks plausible in p50/p95/p99.

The change landed in:

- `scripts/bench_sql_latency.sh`
- `scripts/tests/test_bench_sql_latency_summary.py`

## Validation

- `bash -n scripts/bench_sql_latency.sh`
- `python3 -m unittest scripts.tests.test_bench_sql_latency_summary`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Smoke read

The new launcher regression test extracts the inline summary Python from
`scripts/bench_sql_latency.sh`, feeds it synthetic result files, and checks:

- a negative `cached-plan` sample is rejected with a non-zero exit
- positive `plain-server` samples still summarize normally

## Exit criteria

- negative per-query timings make the launcher fail instead of printing a bogus
  summary
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- shell smoke shows the summary path rejects a negative sample
