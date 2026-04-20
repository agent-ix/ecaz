---
id: FR-027
title: pgrx PG18 Support and Build Configuration
type: functional-requirement
status: DRAFT
object_type: configuration
traces:
  - US-004
  - StR-004
---
# FR-027: pgrx PG18 Support and Build Configuration

## Requirement

The extension SHALL add PostgreSQL 18 as a supported target via pgrx feature flags, making PG18 the default build target while maintaining PG17 compatibility.

Current staged behavior:
- PG18 is now the default Cargo feature and PG17 remains the supported fallback.
- Local validation now covers both versions (`cargo test`, `cargo pgrx test pg18`,
  `cargo pgrx test pg17`, and version-specific clippy lanes), and CI initializes both pg17 and
  pg18 explicitly.
- `_PG_init()` now registers the PG18 EXPLAIN hooks and shared-stats setup. Shared pgstat
  activation still depends on preload-time configuration.
- `scripts/run_pg18_preload_pgstat_test.sh` now covers that preload-only PG18 shared-pgstat path
  in a repo-local cluster, which the ordinary `cargo pgrx test pg18` lane cannot exercise.
- The upgrade/diagnostics/read-stream snapshot helpers now describe live wired state rather than
  pure scaffolding.

### Cargo.toml Changes

Drop PG14-16 (never tested, no users, pgrx template boilerplate). Support PG17 as fallback, PG18 as default:

```toml
[features]
default = ["pg18"]
pg17 = ["pgrx/pg17", "pgrx-tests/pg17"]
pg18 = ["pgrx/pg18", "pgrx-tests/pg18"]

[dependencies]
pgrx = "0.17"
```

### Conditional Compilation Strategy

PG18-specific features SHALL be gated behind `#[cfg(feature = "pg18")]`:

| Feature | PG18 | PG17 |
|---|---|---|
| `read_stream` API | `ReadStream` prefetch | `ReadBufferExtended` sync |
| `amgettreeheight` | Registered callback | Field not set |
| `amtranslatestrategy` | Registered callback | Field not set |
| `amtranslatecmptype` | Registered callback | Field not set |
| `amconsistentordering` | `true` | Field not set |
| Custom EXPLAIN options | Registered hook | Counters only (no hook) |
| Custom pgstat | Registered kind | Not available |
| `PG_MODULE_MAGIC_EXT` | Name + version | Standard magic |

### IndexAmRoutine PG18 Fields

The following fields are new in PG18's `IndexAmRoutine` struct. pgrx bindings SHALL expose them via `pg_sys`. If pgrx does not yet expose them, the implementation SHALL use raw pointer arithmetic or `#[repr(C)]` struct overlay:

```rust
#[cfg(feature = "pg18")]
{
    amroutine.amgettreeheight = Some(ec_hnsw_amgettreeheight);
    amroutine.amtranslatestrategy = Some(ec_hnsw_amtranslatestrategy);
    amroutine.amtranslatecmptype = Some(ec_hnsw_amtranslatecmptype);
    amroutine.amconsistentequality = false;
    amroutine.amconsistentordering = true;
}
```

### CI Matrix

The CI pipeline SHALL test:
- `cargo pgrx test pg18` — all tests including PG18-specific
- `cargo pgrx test pg17` — existing tests pass (PG18-specific tests compiled out)
- PG18-specific tests SHALL be gated with `#[cfg(feature = "pg18")]`
- PG14-16 feature flags SHALL be removed from Cargo.toml (ADR-016)

### `_PG_init` Function

PG18 features (EXPLAIN options, pgstat) require a `_PG_init` entry point. The extension SHALL add:

```rust
#[pg_guard]
pub unsafe extern "C-unwind" fn _PG_init() {
    #[cfg(feature = "pg18")]
    {
        register_explain_option();
        register_pg18_stats();
    }
}
```

## Acceptance Criteria

### FR-027-AC-1: PG18 builds
`cargo pgrx build --features pg18 --release` SHALL succeed.

### FR-027-AC-2: PG17 builds
`cargo pgrx build --features pg17 --release` SHALL succeed with no PG18-specific code compiled.

### FR-027-AC-3: Tests pass on both
`cargo pgrx test pg17` and `cargo pgrx test pg18` SHALL both pass.

### FR-027-AC-4: _PG_init called
On PG18, `CREATE EXTENSION ecaz` SHALL invoke `_PG_init`, registering the EXPLAIN option and any PG18 diagnostics setup that is not still blocked on preload-time pgstat wiring.
