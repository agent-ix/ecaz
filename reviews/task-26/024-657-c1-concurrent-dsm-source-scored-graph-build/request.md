# Review Request: Source-Scored Concurrent DSM Graph Build

## Summary

Please review commit `50290adca464f236eacd05c2ae1f6a6a2ae12639`, which removes the concurrent DSM graph assembly blocker for `build_source_column` indexes.

The implementation now packs source vectors into the concurrent DSM graph image and uses those source vectors for graph candidate scoring when present. Encoded-code scoring remains the fallback for indexes without source vectors.

This is intentionally a graph-assembly slice. Source-scored builds still ingest heap tuples through the existing serial source scan path; this change lets the later DSM graph assembly phase run with worker participants instead of erroring out.

## What Changed

- Added an optional source-vector corpus to `EcHnswConcurrentDsmPreassemblyPlan`.
- Extended the concurrent DSM graph layout/header with `source_dim` and a source-corpus section.
- Copied source vectors into DSM during graph image initialization.
- Switched concurrent DSM graph insertion scoring to source inner product when `source_dim > 0`.
- Replaced the prior source-scored rejection test with coverage that verifies source vectors are packed into the DSM preassembly plan.

## Evidence

Validation passed:

- `cargo test`
- `cargo pgrx test pg18`
- `cargo test -p ecaz build_parallel -- --nocapture`
- `cargo pgrx test pg18 test_pg18_parallel_index_build_concurrent_dsm_graph_opt_in`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Packet-local smoke artifact:

- `artifacts/pg18_source_dsm_smoke.sql`
- `artifacts/pg18_source_dsm_smoke.log`

Key smoke result:

- 2 requested workers, 2 launched workers
- 2000 heap tuples
- 1998 index tuples after duplicate coalescing
- `graph_us = 305961`
- `concurrent_dsm_graph_workers_launched = 2`

## Known Limits

- This packet proves the previous source-scored concurrent DSM graph assembly error is gone on a 2k PG18 SQL smoke fixture.
- It does not rerun the real 50k source-scored recall packet. The prior real-corpus packet remains the next measurement target after reloading the local real-corpus fixture.
- The smoke SQL used `DROP EXTENSION ... CASCADE` in the local scratch database, so the earlier local 50k real-corpus tables need to be reloaded before rerunning packet 656.
