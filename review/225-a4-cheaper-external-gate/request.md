# Review Request: A4 Cheaper External Gate + Real 50K 50-Query Checkpoint

## Context

Branch:
- `main`

Prior relevant packets:
- `review/223-a4-real-10k-pass-and-loader-m-values/request.md`
- `review/224-a4-real-50k-directional-summary/request.md`

The real-corpus lane had already established that:

- canonical real `10K` passes strongly
- real `50K` directional slices look healthy

But the next broader signoff step was too expensive in the current harness. A
`50`-query real `50K` gate run stayed CPU-bound for more than `22` minutes and
never returned a captured result before being stopped.

This slice fixes the avoidable part of that cost and records the resulting
broader real `50K` gate checkpoint.

## What Landed

### 1. External gate runs no longer pay for exact-quantized top-10

The old external gate path shared one helper with the external summary path:

- `build_external_recall_context(...)`

That helper always computed:

- brute-force fp32 truth, and
- exact quantized top-10 via `ORDER BY embedding <#> $1 LIMIT 10`

even when the caller only wanted the plain gate report from
`tqhnsw_graph_scan_recall_external_gate_report(...)`.

That was unnecessary. The gate report only needs graph Recall@10 against fp32
truth; it does not emit exact-quantized overlap metrics.

The new split in [src/lib.rs](/home/peter/dev/tqvector/src/lib.rs):

- keeps the full external summary path unchanged
- makes `build_external_recall_context(..., include_exact_quantized_top10)`
  conditional
- adds a cheaper gate-only path,
  `probe_graph_scan_recall_external_gate_row_for_context(...)`, that skips
  exact-quantized top-10 entirely

So:

- `tqhnsw_graph_scan_recall_external_summary(...)` still pays for exact-vs-graph
  metrics
- `tqhnsw_graph_scan_recall_external_gate_report(...)` no longer does

### 2. Long real-corpus gate runs now have a detached scratch helper

Added [run_real_corpus_recall_scratch.sh](/home/peter/dev/tqvector/scripts/run_real_corpus_recall_scratch.sh).

Purpose:

- run external gate or summary queries against the scratch `pg17` cluster
- optionally detach the run so it survives client/session teardown
- write SQL, result TSV, and log files under
  `/home/peter/dev/tqvector/tmp/real_corpus_runs`

Important implementation detail:

- detached mode uses server-side `COPY (<query>) TO '...tsv'`

That avoids the earlier problem where a long-running `psql` client session could
die before the result was captured.

### 3. Broader real `50K` gate checkpoint now completes cleanly

With the cheaper gate path plus detached capture, the broader real `50K` gate
over `50` real queries completed successfully and wrote:

- `/home/peter/dev/tqvector/tmp/real_corpus_runs/20260410T235725Z_gate_tqhnsw_real_50k_tqhnsw_real_50k_queries_50.tsv`

Observed output:

```text
8   40   0.926       t
8   128  0.944  0.89 t
8   200  0.948       t
16  200  0.952       t
```

This is materially stronger than the earlier `10`/`25`-query directional read
and it still clears the A4 gate comfortably at the required point:

- `(m=8, ef_search=128)`: `94.4%`

## Evidence

### Failed path: old broader gate run stayed hot for more than `22` minutes

Before the fix, the detached real `50K` gate run over `50` queries remained:

- CPU-bound
- in `copy (select * from tests.tqhnsw_graph_scan_recall_external_gate_report(...`
- with no result file written

for more than `22` minutes before it was stopped.

The concrete code reason was visible in [src/lib.rs](/home/peter/dev/tqvector/src/lib.rs):

- `run_graph_scan_recall_gate_from_external(...)` called
  `build_external_recall_context(...)`
- `build_external_recall_context(...)` always executed the exact-quantized SPI
  top-10 query loop even though the gate report never used those rows

### New broader real `50K` gate result

Observed output from:

```sql
select * from tests.tqhnsw_graph_scan_recall_external_gate_report(
    'tqhnsw_real_50k_corpus',
    'tqhnsw_real_50k_queries_50',
    'tqhnsw_real_50k'
);
```

captured via the detached helper was:

```text
8   40   0.926       t
8   128  0.944  0.89 t
8   200  0.948       t
16  200  0.952       t
```

The output file timestamp shows completion at roughly `3` minutes after launch,
which is a useful step down from the earlier `22+` minute stalled run.

## Readout

The live A4 picture is now stronger and cleaner:

- real `10K` already passed strongly
- real `50K` directional slices already looked healthy
- the broader real `50K` `50`-query gate now also passes strongly

At this point, the remaining A4 question is mostly process/signoff:

- whether this `50`-query real `50K` gate checkpoint is sufficient to call A4
  done, or
- whether the project still wants one more broader real-corpus artifact

The main harness lesson is also clear:

- exact-vs-graph summary should stay expensive and explicit
- plain gate reporting must stay on the cheaper graph-vs-fp32 path only

## Files

- [src/lib.rs](/home/peter/dev/tqvector/src/lib.rs)
- [run_real_corpus_recall_scratch.sh](/home/peter/dev/tqvector/scripts/run_real_corpus_recall_scratch.sh)
