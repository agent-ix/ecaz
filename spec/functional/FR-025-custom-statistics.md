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
- On PG18, `ecaz_stats()` is live.
- When `ecaz` is loaded through `shared_preload_libraries`, `_PG_init()` registers the custom
  pgstat kind through the preload-only C shim and the SQL surface reads the shared snapshot.
- `scripts/run_pg18_preload_pgstat_test.sh` validates that preload lane by starting a repo-local
  PG18 cluster, forcing `shared_preload_libraries = 'ecaz'`, and checking that shared counters are
  visible across backend boundaries.
- In ordinary non-preloaded PG18 sessions, the same SQL surface falls back to backend-local
  counters and diagnostics continue to report `pg18_pgstat_kind_ready = false`.
- Read-only diagnostics snapshot helpers still expose the EXPLAIN-and-pgstat readiness boundary in
  one place.
- PG17 still omits both the SQL function and custom pgstat registration.
- Reset support for custom kinds remains blocked in the local PG18 tree because
  `pg_stat_reset_shared(text)` does not accept custom kind names.

### PG18 Custom Statistics API

PG18 introduces `pgstat_register_kind()` which allows extensions to register custom statistics types that integrate with PostgreSQL's standard statistics infrastructure.

### Statistics Structure

```rust
#[repr(C)]
struct EcazStats {
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
static ECAZ_STATS_KIND: PgStat_KindInfo = PgStat_KindInfo {
    fixed_amount: true,
    accessed_across_databases: true,
    write_to_file: true,
    shared_size: size_of::<TqVectorSharedStats>(),
    name: "ecaz",
    // ... callbacks
};

const ECAZ_PGSTAT_KIND: PgStat_Kind = PGSTAT_KIND_CUSTOM_MIN + 1;

pgstat_register_kind(ECAZ_PGSTAT_KIND, &ECAZ_STATS_KIND);
```

### SQL Interface

```sql
-- Read current statistics
SELECT * FROM ecaz_stats();

-- Returns:
--   total_distance_calcs  | 1234567
--   total_graph_hops      | 45678
--   total_linear_pages    | 890
--   total_scans           | 1234
--   bootstrap_hit_rate    | 0.85
--   quantizer_cache_rate  | 0.99

-- Resetting the custom kind currently requires PostgreSQL's pgstat reset API
-- for the registered kind. The built-in pg_stat_reset_shared(text) SQL helper
-- in the local PG18 tree does not yet accept custom kind names.
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

On PG17, the custom statistics API does not exist. The extension SHALL not register any pgstat
kind, `ecaz_stats()` SHALL NOT be defined, and counter increments SHALL be compiled out. On
PG18, the shared pgstat path requires preload-time activation; without preload, the SQL surface
falls back to backend-local counters.

## Acceptance Criteria

### FR-025-AC-1: Stats function exists
On PG18, `SELECT * FROM ecaz_stats()` SHALL return a row with all defined counters.

### FR-025-AC-2: Counters increment
After running 10 HNSW scan queries, `total_scans_started` SHALL be ≥ 10 and `total_distance_calcs` SHALL be > 0.

### FR-025-AC-3: Reset blocker documented
Until PostgreSQL exposes a reset surface for custom pgstat kinds in this environment, Ecaz
SHALL document the limitation rather than claim that `pg_stat_reset_shared(text)` can reset the
custom kind directly.

### FR-025-AC-4: Persistence within session
Counters SHALL accumulate across queries within a session. They SHALL NOT reset between queries.

### FR-025-AC-5: PG17 graceful absence
On PG17, calling `ecaz_stats()` SHALL raise an appropriate error or the function SHALL not exist.

## References

- PG source: `src/include/utils/pgstat_internal.h` — `pgstat_register_kind()`, `PgStat_KindInfo` struct (all fields and callbacks), `pgstat_get_custom_shmem_data()`, `pgstat_get_custom_snapshot_data()`
- PG source: `src/include/utils/pgstat_kind.h` — `PGSTAT_KIND_CUSTOM_MIN` (24), `PGSTAT_KIND_CUSTOM_MAX` (32), `PGSTAT_KIND_EXPERIMENTAL` (24, for dev use)
- PG source: `src/backend/utils/activity/pgstat.c` — registration flow, shmem allocation, snapshot/flush lifecycle
- PG source: `src/backend/utils/activity/pgstat_shmem.c` — shared memory backing for custom stat entries
