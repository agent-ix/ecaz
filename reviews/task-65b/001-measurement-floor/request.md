# Task 65b Review Request: Measurement Floor

## Scope

This opens Task 65b with Slice A only: a current-head measurement floor for
DiskANN parallel Vamana graph-construction planning. There is no code change in
this packet.

The checked-in suite config is `suite.json` and uses `ecaz bench suite` per
FR-038. It measures the current single-process `ec_diskann` path with:

- `storage_format = 'pq_fastscan'`
- `graph_degree=32`
- `build_list_size=100`
- `alpha=1.2`
- isolated one-index-per-table prefixes
- real10k and real100k local M5 fixtures

The goal is to capture enough phase data to make the next Task 65b design call
concrete: shared neighbour cache shape, locking strategy, and whether the first
parallel worker slice should use PostgreSQL `ParallelContext` directly or a
temporary rayon stepping stone.

## Result Summary

Final suite command:

```text
./target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-65b/001-measurement-floor/suite.json --manifest-output reviews/task-65b/001-measurement-floor/artifacts/suite-manifest.json --results-output reviews/task-65b/001-measurement-floor/artifacts/results.jsonl
```

The final 8-step suite passed. Key rows:

| fixture | SQL index build | total load | recall@10 L64/L128/L200 | DiskANN index size |
| --- | ---: | ---: | --- | ---: |
| real10k R32/L100 | `6.72s` | `9.50s` | `0.9965 / 0.9970 / 0.9975` | `4.7 MiB` |
| real100k R32/L100 | `243.29s` | `271.26s` | `0.9190 / 0.9640 / 0.9755` | `46.1 MiB` |

The optimized real10k in-memory Vamana build probe recorded:

- `build_seconds=62.045`
- `reachable_fraction=1.000000`
- pass 1 visited mean/p95 `101.30/104`
- selected mean/p95 `20.37/32`
- backlinks `142105`
- reprunes `61593`
- in-degree p95/p99/max `52/79/2881`

The suite intentionally does not run the real100k in-memory build probe. The
SQL build already shows real100k wall time, and the 10k probe took ~62s even in
the optimized CLI; extrapolating that probe to 100k would be disproportionate
for a Slice A packet.

## Evidence

- `suite-manifest.json`
- `results.jsonl`
- `manifest.md`
- `precheck-host.log`
- load logs containing loader-level SQL build timing
- `bench diskann-build-probe` log for real10k in-memory Vamana diagnostics
- recall logs for L=`64,128,200`
- storage logs
- release backend install log

The local checkout currently has real10k and real100k fixtures. I did not find
a local synth10k fixture in `data/` or `fixtures/`; if one is restored, a
follow-up run can add the synth10k row without changing the packet shape.

One measurement-surface limitation: `ec_diskann_ambuild_timing` is emitted with
`pgrx::notice!`, but the `corpus load` path uses the async Postgres client and
does not mirror backend notices into the load log. The durable phase/counter
detail in this packet is therefore the real10k `bench diskann-build-probe`
output; real100k evidence is loader-level SQL build time plus recall/storage.

## Review Focus

- Whether this is the right Task 65b Slice A measurement floor before code.
- Whether the suite captures enough phase/counter shape and recall/storage
  guardrails for the Slice B/C design decisions.
- Whether real10k plus real100k is enough to proceed while synth10k is absent
  locally.
