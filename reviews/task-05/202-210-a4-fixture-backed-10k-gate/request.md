# Review Request: A4 Fixture-Backed 10K Gate Helpers

Basis: `main` working tree after review `209`

## Why This Packet Exists

Review `209` narrowed the current `4+0` lane further, but the live A4 loop still had a practical
problem:

- the real `10K` graph-first gate was expensive enough that a rerun could burn most of a turn
- repeated reruns rebuilt the same synthetic corpus and indexes from scratch
- that made it hard to separate "report calculation is cheap" from "fixture reset is still too
  slow"

This packet lands the small harness split needed to make that boundary explicit.

## Landed Helper Surfaces

New SQL/debug surfaces in [src/lib.rs](/home/peter/dev/tqvector/src/lib.rs):

- `tqhnsw_graph_scan_recall_fixture_gate_reset(fixture_prefix, corpus_size)`
- `tqhnsw_graph_scan_recall_fixture_gate_report(fixture_prefix, query_count)`

They expose the `10K` A4 path as two phases:

1. one-time fixture reset/build
2. reusable gate report reads over the existing built indexes

There is also a new ignored pg-test:

- `test_tqhnsw_graph_scan_recall_fixture_gate_10k_tiled_fwht`

That test exists only to time the reset and then prove that two consecutive gate reports return the
same result set on the same fixtures.

## Important Implementation Change

The first helper version still duplicated the synthetic corpus:

- one table for `m=8`
- one table for `m=16`

That was unnecessary work for the same A4 corpus. The landed version now shares one corpus table:

- `<prefix>_corpus`

and builds two indexes on top of it:

- `<prefix>_m8_idx`
- `<prefix>_m16_idx`

So the helper now avoids paying corpus generation/load twice while keeping the separate index
operating points needed by A4.

## Experiments Run

### 1. Initial duplicated-corpus timing attempt

Command:

```bash
PGRX_HOME=/tmp/tqvector_pgrx_home cargo test --no-default-features --features 'pg17 pg_test' \
  tests::pg_test_tqhnsw_graph_scan_recall_fixture_gate_10k_tiled_fwht \
  -- --exact --ignored --nocapture
```

Observed behavior:

- test entered `pg_stat_progress_create_index`
- phase remained `building index`
- elapsed time passed `21m`
- the test never reached the first reusable report phase before being stopped

Readout:

- the helper split was valid
- but duplicating the `10K` corpus per `m` was obviously the wrong shape

### 2. Revised shared-corpus timing attempt

Same command after switching to one shared corpus table plus two indexes.

Observed behavior:

- the test again entered `pg_stat_progress_create_index`
- phase remained `building index`
- elapsed time passed `10m50s`
- the test still had not reached the first reusable report before being stopped

Readout:

- the shared-corpus change removed real duplicated load work
- but the dominant remaining long pole is now clearly one-time `CREATE INDEX`
- repeated report reuse is no longer the questionable part of the harness

## Secondary Validation Note

One full `cargo test` run transiently failed
`test_tqhnsw_graph_first_scan_emits_distance_sorted_scores` immediately after an aborted timing
probe. That did **not** reproduce:

- the targeted isolated rerun passed
- the subsequent full `cargo test` rerun passed
- the full `cargo pgrx test pg17` rerun passed

So this was treated as stale pg-test state after killing the long-running probe, not as a new
runtime regression from the harness patch.

## What This Changes

This slice does not change A4 recall results.

What it does change:

1. the repo can now separate fixture reset from reusable A4 gate reports
2. the helper no longer loads the same `10K` corpus twice for `m=8` and `m=16`
3. the remaining harness bottleneck is now isolated: one-time index build cost

That is useful because it narrows the next harness work substantially. If we want an interactive
`10K` A4 loop, the next improvement has to attack one of:

- index build cost directly
- fixture persistence outside the per-test transaction lifecycle
- or a cheaper way to keep built `10K` fixtures warm across reruns

It is no longer worth spending another slice on report-side plumbing alone.

## Validation

Required validation on the landed shared-corpus helper path:

```bash
cargo test
PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
```

All three passed.

## Commands Run

```bash
cargo test
PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17
cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings
PGRX_HOME=/tmp/tqvector_pgrx_home cargo test --no-default-features --features 'pg17 pg_test' \
  tests::pg_test_tqhnsw_graph_scan_recall_fixture_gate_10k_tiled_fwht \
  -- --exact --ignored --nocapture
PGRX_HOME=/tmp/tqvector_pgrx_home cargo test --no-default-features --features 'pg17 pg_test' \
  tests::pg_test_tqhnsw_graph_first_scan_emits_distance_sorted_scores \
  -- --exact --nocapture
```

Monitoring commands used during the timing probes:

```bash
/home/peter/.pgrx/17.9/pgrx-install/bin/psql -h /home/peter/dev/tqvector/target/test-pgdata -p 40217 -d postgres -At -F $'\t' \
  -c "select state, wait_event_type, wait_event, left(query, 120) from pg_stat_activity where datname = 'pgrx_tests' and state <> 'idle';"
/home/peter/.pgrx/17.9/pgrx-install/bin/psql -h /home/peter/dev/tqvector/target/test-pgdata -p 40217 -d postgres -At -F $'\t' \
  -c "select pid, phase, lockers_total, lockers_done, blocks_total, blocks_done, tuples_total, tuples_done from pg_stat_progress_create_index;"
```

## Review Focus

- whether the shared-corpus fixture shape is the right default for reusable `10K` A4 reruns
- whether stopping the timing probes at `>21m` and `>10m50s` was the right call for this lane
- whether the next harness step should target index build cost directly rather than more report
  plumbing
