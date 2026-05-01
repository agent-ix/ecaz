---
id: ADR-016
title: PostgreSQL 18 as Primary Target
status: DECIDED
date: 2026-04-06
---
# ADR-016: PostgreSQL 18 as Primary Target

## Context

Ecaz was built targeting PG17. PostgreSQL 18 (released 2025-09-25) introduces:

1. **Async I/O subsystem** — `ReadStream` API with `io_method=sync|worker|io_uring`. Thomas Munro's prototype on pgvector HNSW showed 4x cold-cache speedup.
2. **New IndexAmRoutine callbacks** — `amgettreeheight`, `amtranslatestrategy`, `amtranslatecmptype` for planner integration.
3. **Custom EXPLAIN options** — `RegisterExtensionExplainOption` for per-query diagnostics.
4. **Custom cumulative statistics** — `pgstat_register_kind` for aggregate operational metrics.
5. **GIN parallel build** — Reusable `ParallelContext` + `Sharedsort` infrastructure for parallel index construction.
6. **`PG_MODULE_MAGIC_EXT`** — Extension name/version reporting.

At the time this ADR was written, Ecaz had several production blockers that PG18 features directly addressed:
- The cost model was disabled by a prohibitive-cost gate.
- Page reads were synchronous, which hurt cold-cache performance on network storage.
- Vacuum was incomplete for the target index lifecycle.
- Index build was serial and did not scale for larger tables.

### PG17 Fallback Complexity

The fallback cost is minimal (~65 lines of `#[cfg]` guards) because the current codebase IS the PG17 path:
- `routine.rs`: ~15 lines gating new `IndexAmRoutine` fields (pgrx handles struct layout per PG version)
- `scan.rs`/`graph.rs`: ~50 lines of dispatch wrappers for `read_stream` vs `ReadBufferExtended`
- `explain.rs`, `stats.rs`: entire PG18-only modules — PG17 doesn't compile them, zero branching
- Cost model, vacuum, parallel build: work on both versions, no `cfg` needed

## Decision

Adopt PostgreSQL 18 as the primary build target and default feature. PG17 remains supported via feature-flag fallback. Drop PG14-16 support — those versions are EOL or near-EOL and add CI burden without real users.

## Consequences

### Positive
- 3-4x cold-cache scan improvement via `read_stream` prefetch
- Planner-driven index selection via real cost model + `amgettreeheight`
- Production-grade diagnostics via custom EXPLAIN and pgstat
- Parallel build scales with available cores
- Extension version visible via `pg_get_loaded_modules()`

### Negative
- Shared pgstat activation on PG18 still requires `shared_preload_libraries = 'ecaz'` plus a restart
- Conditional compilation (`#[cfg(feature = "pg18")]`) still adds maintenance complexity
- PG17 users do not get async I/O, EXPLAIN hooks, or shared pgstat integration
- Current `pgrx 0.17` PG18 support still needs a repo-local explicit `pg_module_magic!` field assignment and a small C shim over `pgstat_internal.h`

### Neutral
- Graph construction in parallel build remains serial (the native builder is still leader-only)
- `read_stream_reset()` distance reset behavior may require workarounds for HNSW burst patterns

## References

### PostgreSQL 18 Release
- [PostgreSQL 18 Release Notes](https://www.postgresql.org/docs/18/release-18.html)
- [PostgreSQL 18 AIO Deep Dive — credativ](https://www.credativ.de/en/blog/postgresql-en/postgresql-18-asynchronous-disk-i-o-deep-dive-into-implementation/)
- [Waiting for Postgres 18: Accelerating Disk Reads with Async I/O — pganalyze](https://pganalyze.com/blog/postgres-18-async-io)

### ReadStream API (Primary Source)
- PG source: `src/include/storage/read_stream.h` — callback type, creation, consumption, flags
- PG source: `src/backend/storage/aio/read_stream.c` — adaptive prefetch, combined I/O, fast path
- PG source: `src/backend/storage/aio/README` — AIO subsystem design rationale
- [Thomas Munro: Trying out read streams in pgvector — pgsql-hackers](https://www.mail-archive.com/pgsql-hackers@lists.postgresql.org/msg171681.html) — prototype showing 4x speedup on HNSW
- [Thomas Munro: Follow-up patches and benchmarks](https://www.mail-archive.com/pgsql-hackers@lists.postgresql.org/msg178397.html) — `reset_distance` issue, burst pattern analysis
- [BitmapHeapScan read_stream conversion commit](https://www.mail-archive.com/pgsql-committers@lists.postgresql.org/msg39237.html) — reference for converting an existing scan to read_stream

### Index AM API Changes
- PG source: `src/include/access/amapi.h` — `IndexAmRoutine` struct, `amgettreeheight`, `amtranslatestrategy`, `amtranslatecmptype`
- PG source: `src/include/access/cmptype.h` — `CompareType` enum definition
- PG source: `src/backend/access/nbtree/nbtree.c` — btree reference implementations of all three callbacks
- PG source: `src/backend/optimizer/util/plancat.c` — how planner calls `amgettreeheight` and stores result in `IndexOptInfo.tree_height`

### Custom EXPLAIN Options
- PG source: `src/include/commands/explain_state.h` — `RegisterExtensionExplainOption`, `ExplainOptionHandler`, extension state API
- PG source: `src/include/commands/explain_format.h` — `ExplainPropertyInteger`, `ExplainPropertyBool`, output helpers
- PG source: `src/backend/commands/explain.c` — `explain_per_node_hook`, `explain_per_plan_hook`

### Custom Cumulative Statistics
- PG source: `src/include/utils/pgstat_internal.h` — `pgstat_register_kind`, `PgStat_KindInfo` struct, callback types
- PG source: `src/include/utils/pgstat_kind.h` — `PGSTAT_KIND_CUSTOM_MIN`, `PGSTAT_KIND_EXPERIMENTAL`
- PG source: `src/backend/utils/activity/pgstat.c` — registration flow, shmem allocation, snapshot/flush

### GIN Parallel Build (Pattern Reference)
- PG source: `src/backend/access/gin/gininsert.c` — `_gin_begin_parallel`, `_gin_parallel_build_main`, shared memory keys, sort coordination
- PG source: `src/backend/access/gin/ginutil.c` — `amcanbuildparallel = true` flag

### PG_MODULE_MAGIC_EXT
- PG source: `src/include/fmgr.h` — `Pg_magic_struct` with `.name` and `.version` fields, macro definition

### PostgreSQL AIO Wiki
- [PostgreSQL AIO Wiki](https://wiki.postgresql.org/wiki/AIO) — design overview, io_method comparison, performance results

### Buffer Management (PG18 additions)
- `pg_buffercache_evict_relation()` — evict all buffers for a relation (useful for cold-cache benchmarking)
- `pg_buffercache_evict_all()` — evict all shared buffers
- NUMA awareness: `pg_buffercache_numa` view, `--with-libnuma` build option
