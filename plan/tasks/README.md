# Task Breakdown

These task files are the parallel execution breakdown for `tqvector`.

## Recommended start order

1. `01-quantizer-core.md`
2. `02-datum-and-io.md`
3. `04-page-layout-and-wal.md`
4. `08-safety-and-ci.md`

## Start after foundations stabilize

1. `03-sql-surface.md`
2. `05-build-and-scan.md`

## Start after indexed query path exists

1. `06-vacuum-and-insert.md`
2. `07-simd-and-benchmarks.md`

## Coordination rules

- Freeze binary datum layout before downstream work expands.
- Freeze `ProdQuantizer` scoring interfaces before SIMD work begins.
- Freeze page tuple and WAL helper APIs before build, vacuum, and insert proceed independently.
- Keep benchmark work off the critical path until correctness is stable.
