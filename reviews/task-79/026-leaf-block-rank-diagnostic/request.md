# Task 79 Review Request: Leaf Block Rank Diagnostic

## Summary

This packet adds a SQL-visible diagnostic and `ecaz bench spire-pipeline` JSONL output for exact target block ranks under the production SPIRE RaBitQ routed-leaf block selector.

The diagnostic was run locally only. No AWS commands were used.

Code checkpoint:

- `e7a956cd4` - `Add SPIRE leaf block rank diagnostic`

Packet artifacts:

- `artifacts/manifest.md`
- `artifacts/leaf-block-rank-analysis.md`
- `artifacts/leaf-block-rank-100k-rabitq-global384-rw0.jsonl`
- `artifacts/pipeline-leaf-block-rank-100k-rabitq-global384-rw0.log`

## What Changed

- Added `ec_spire_index_scan_leaf_block_rank_snapshot(index_oid, query, target_local_sequences)` to report exact target local vec-id block rank, selected-by-cap status, routed leaf placement, row range, row index, block score, and missing status.
- Added `--leaf-block-rank-output` to `ecaz bench spire-pipeline`.
- Added suite config support for the new output.

## Diagnostic Result

Run shape:

- RaBitQ, local PG18
- `task79_surface_100k`
- 200 queries
- `nprobe=96`
- `rerank_width=25`
- clustered block64 summaries
- global block cap 384
- radius weight 0.0

Pipeline result:

| candidates | p50 | recall@10 |
| ---: | ---: | ---: |
| 4,764,181 | 43.218 ms | 0.9690 |

Block-rank result over 2,000 exact top-10 targets:

| status | count |
| --- | ---: |
| `block_ranked` | 1,995 |
| `not_found_in_routed_leaves` | 5 |

At cap 384, 1,938 exact top-10 targets are selected and 62 are missed. That exactly matches 0.9690 recall, so the recall failure is explained by block selection before rerank.

## Cap Readout

Using the same rank file:

| global cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 384 | 1,938 | 62 |
| 416 | 1,944 | 56 |
| 512 | 1,965 | 35 |
| 640 | 1,979 | 21 |
| 768 | 1,986 | 14 |
| 1024 | 1,994 | 6 |

The Task 79 recall gate needs at most 15 misses over 2,000 exact top-10 targets. This rank distribution reaches that only around cap 768, and packet 025 already measured cap 768 at 9,525,502 candidates and p50 56.486 ms. That violates the candidate and latency gates.

## Readout

This rules out "just bump to 400 or 416" as the fix. Only 5 targets are not in the routed leaves; 57 targets are in routed leaves but ranked below cap 384 by the current block score.

The next Task 79 implementation should improve per-block information content or block-score discrimination at the same 384-416 candidate budget. The evidence points away from wider caps and away from deterministic one/two-row sampling as the primary solution.

## Validation

- `cargo fmt --check`: passed.
- `cargo check -p ecaz --no-default-features --features pg18`: passed.
- `cargo check -p ecaz-cli`: passed with the existing `LoadedDistributedPlacementConfig.path` dead-code warning.
- `cargo build -p ecaz-cli`: passed with the same existing warning.
- Local PG18 install and restart: passed.
- Suite audit and local suite run: passed.
