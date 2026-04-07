---
id: FR-025
title: Custom Cumulative Statistics
type: functional-requirement
status: DRAFT
object_type: entity
traces:
  - US-011
  - StR-004
---
# FR-025: Custom Cumulative Statistics

## Requirement

On PG18, the extension SHALL register a custom pgstat kind to track aggregate operational metrics across all queries, visible via a SQL function and resettable via standard PostgreSQL statistics reset.

Current staged behavior:
- Before PostgreSQL 18 support exists in this repository, pure statistics-scaffolding helpers MAY
  expose the intended SQL function name and report that both pgstat-kind registration and SQL
  function wiring remain unavailable.
- The staged implementation MAY also define a reusable cumulative-stats struct in planner-owned
  code so the runtime lane can increment the intended metrics and the future PG18 pgstat glue can
  flush them into PostgreSQL's statistics infrastructure without requiring this branch to edit
  `scan.rs`.
- Those same helpers MAY also define pure summary logic for the derived SQL-facing rates shown
  below, including `bootstrap_hit_rate` and `quantizer_cache_rate`, without implying that
  `tqvector_stats()` exists on PG17.
- Read-only diagnostics snapshot helpers MAY also expose the current EXPLAIN-and-pgstat readiness
  state together so productization work can inspect one consolidated PG18 diagnostics boundary.
- Those helpers SHALL stay descriptive only; they do not imply that `tqvector_stats()` exists on
  PG17 or that any counters are being accumulated through PostgreSQL's statistics system.

### PG18 Custom Statistics API

PG18 introduces `pgstat_register_kind()` which allows extensions to register custom statistics types that integrate with PostgreSQL's standard statistics infrastructure.

### Statistics Structure

```rust
#[repr(C)]
struct TqVectorStats {
    total_distance_calcs: i64,       // Total score_ip_from_parts calls
    total_graph_hops: i64,           // Total bootstrap expansion node visits
    total_linear_pages: i64,         // Total linear scan pages read
    total_scans_started: i64,        // Total amrescan calls
    total_scans_bootstrap_only: i64, // Scans that returned all results from bootstrap
    quantizer_cache_hits: i64,       // ProdQuantizer cache hits
    quantizer_cache_misses: i64,     // ProdQuantizer cache misses (new codebook)
}
```

### Registration

In `_PG_init()`:

```rust
static TQVECTOR_STATS_KIND: PgStat_KindInfo = PgStat_KindInfo {
    fixed_amount: true,
    accessed_across_databases: true,
    write_to_file: true,
    shared_size: size_of::<TqVectorSharedStats>(),
    name: "tqvector",
    // ... callbacks
};

pgstat_register_kind(PGSTAT_KIND_EXPERIMENTAL, &TQVECTOR_STATS_KIND);
```

### SQL Interface

```sql
-- Read current statistics
SELECT * FROM tqvector_stats();

-- Returns:
--   total_distance_calcs  | 1234567
--   total_graph_hops      | 45678
--   total_linear_pages    | 890
--   total_scans           | 1234
--   bootstrap_hit_rate    | 0.85
--   quantizer_cache_rate  | 0.99

-- Reset statistics
SELECT pg_stat_reset_shared('tqvector');
```

### Counter Increment Points

| Counter | Increment Location | When |
|---|---|---|
| `total_distance_calcs` | `score_scan_element_result()` | Each element scored |
| `total_graph_hops` | `refill_candidate_frontier_from_source()` | Each node expanded |
| `total_linear_pages` | `next_linear_scan_heap_tid()` | Each page read in linear scan |
| `total_scans_started` | `amrescan()` | Each scan started |
| `total_scans_bootstrap_only` | `amgettuple()` | First call that falls through to linear scan (negated) |
| `quantizer_cache_hits` | `ProdQuantizer::cached()` | Cache hit |
| `quantizer_cache_misses` | `ProdQuantizer::cached()` | Cache miss |

### PG Version Compatibility

On PG17, the custom statistics API does not exist. The extension SHALL not register any pgstat kind. The `tqvector_stats()` function SHALL NOT be defined. Counter increments SHALL be compiled out.
During the current staged implementation, a reusable planner-owned counter struct may exist in
`am/stats.rs`, pure summary helpers for the intended derived rates may also exist there, but no
PostgreSQL pgstat kind is registered and no SQL-visible cumulative statistics are exposed.

## Acceptance Criteria

### FR-025-AC-1: Stats function exists
On PG18, `SELECT * FROM tqvector_stats()` SHALL return a row with all defined counters.

### FR-025-AC-2: Counters increment
After running 10 HNSW scan queries, `total_scans_started` SHALL be ≥ 10 and `total_distance_calcs` SHALL be > 0.

### FR-025-AC-3: Reset works
After `SELECT pg_stat_reset_shared('tqvector')`, all counters SHALL be zero.

### FR-025-AC-4: Persistence within session
Counters SHALL accumulate across queries within a session. They SHALL NOT reset between queries.

### FR-025-AC-5: PG17 graceful absence
On PG17, calling `tqvector_stats()` SHALL raise an appropriate error or the function SHALL not exist.

## References

- PG source: `src/include/utils/pgstat_internal.h` — `pgstat_register_kind()`, `PgStat_KindInfo` struct (all fields and callbacks), `pgstat_get_custom_shmem_data()`, `pgstat_get_custom_snapshot_data()`
- PG source: `src/include/utils/pgstat_kind.h` — `PGSTAT_KIND_CUSTOM_MIN` (24), `PGSTAT_KIND_CUSTOM_MAX` (32), `PGSTAT_KIND_EXPERIMENTAL` (24, for dev use)
- PG source: `src/backend/utils/activity/pgstat.c` — registration flow, shmem allocation, snapshot/flush lifecycle
- PG source: `src/backend/utils/activity/pgstat_shmem.c` — shared memory backing for custom stat entries
