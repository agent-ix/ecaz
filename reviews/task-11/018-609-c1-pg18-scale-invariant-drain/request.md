# PG18 Contributor Drain — Scale-Invariant Diagnostic

## Summary

This packet records a scaled diagnostic run at `rows=5000 limit=100` using the
current HEAD (`3fb2073`) to determine whether the `NoVisibleOwner: 4` result
from the 512-row / limit=16 fixture is fixture-size-specific or structural.

No source changes are included. This is a pure measurement artifact packet.

## Question

Packet 607's feedback identified `NoVisibleOwner: 4` at exit time — the
emitter finishes before contributor hidden rows can drain. The question:

> Is this because the 512-row fixture is too small, or because the emitter
> always finishes before contributors can publish, regardless of fixture size?

## Result

With `rows=5000 limit=100 ef_search=1000 workers=4`, the pattern is identical:

```text
Limit (actual time=38.702..40.067 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 4
Parallel Contributor Poll Limit: Missing Hidden: 0
Parallel Contributor Poll Limit: Duplicate Active: 0
Parallel Contributor Poll Limit: Handoff Ready: 0
Parallel Contributor Poll Limit: Ordered After Visible: 0
Parallel Contributor Poll Limit: No Visible Owner: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Default lane: 18.5ms, 101 expansions, all counters at 0, validation passes.

## Interpretation

The `NoVisibleOwner: 4` pattern is scale-invariant for the current 4D
deterministic fixture. Two root causes combine:

1. **Graph diameter**: In a low-dimensional space with deterministic patterns,
   the HNSW graph has very small effective diameter. All workers starting from
   the same entry point converge to the same top candidates within a few
   traversal steps. Bootstrap seed diversification (from `ee9b405`) does not
   produce different neighborhoods.

2. **Timing**: The emitter traverses all needed candidates and exits before
   contributors can drain. Even with 5000 rows, the 4D vector space is trivially
   small — the emitter explores exactly `limit+1` nodes (101 for limit=100).

The handoff model is theoretically sound. `best_hidden_local_only_blocker_locked`
correctly probes for contributor hidden rows on every candidate check. The drain
mechanism fires if contributors publish before the emitter processes their rows.
But with a 4D deterministic fixture, contributors and the emitter find exactly
the same candidates in the same order.

## Next Step

The correct next test is with a higher-dimensional randomized fixture:
- `--dimensions 16` or higher
- Randomized embeddings (not deterministic modular arithmetic)
- `--rows 50000`, `--limit 100`, `ef_search 500`

At that scale, bootstrap seed diversification is expected to produce genuinely
different graph traversal paths, and contributors might find candidates that the
emitter's early traversal misses.

The `--dimensions` flag needs to be added to the `pg18-parallel-scan` CLI command
before this test can be run.

## Artifacts

- `artifacts/pg18-parallel-5k-default.log`: rows=5000, limit=100, default lane
- `artifacts/pg18-parallel-5k-diagnostic.log`: rows=5000, limit=100, contributor diagnostic lane

## Validation

- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 5000 --limit 100 --log-output target/pg18-parallel-5k-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 5000 --limit 100 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-5k-diagnostic.log`
