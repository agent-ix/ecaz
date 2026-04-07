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
- Before the pgrx/toolchain upgrade lands, read-only upgrade snapshot helpers MAY report the
  current default feature, whether a `pg18` Cargo feature exists, and whether PG18 default-build
  readiness is still pending.
- Read-only diagnostics snapshot helpers MAY also report consolidated EXPLAIN/pgstat readiness so
  the broader PG18 productization boundary is queryable before toolchain work lands.
- Read-only ReadStream snapshot helpers MAY also report the intended graph-versus-linear stream
  modes and keep callback/scan/vacuum readiness explicitly false until PG18 support lands.
- Those helpers SHALL stay descriptive only; they do not imply that PG18 builds, tests, or default
  feature selection already work.

### Cargo.toml Changes

Drop PG14-16 (never tested, no users, pgrx template boilerplate). Support PG17 as fallback, PG18 as default:

```toml
[features]
default = ["pg18"]
pg17 = ["pgrx/pg17", "pgrx-tests/pg17"]
pg18 = ["pgrx/pg18", "pgrx-tests/pg18"]

[dependencies]
pgrx = "0.18"   # or whichever version adds pg18 support
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
    amroutine.amgettreeheight = Some(tqhnsw_amgettreeheight);
    amroutine.amtranslatestrategy = Some(tqhnsw_amtranslatestrategy);
    amroutine.amtranslatecmptype = Some(tqhnsw_amtranslatecmptype);
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
        register_pgstat_kind();
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
On PG18, `CREATE EXTENSION tqvector` SHALL invoke `_PG_init`, registering the EXPLAIN option and pgstat kind.
