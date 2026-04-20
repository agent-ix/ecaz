---
id: ADR-004
title: "Raw pg_sys FFI for IndexAmRoutine with local helper module"
status: DECIDED
impact: HIGH for FR-008, FR-009, FR-010 (entire HNSW AM)
date: 2026-04-03
---
# ADR-004: Raw pg_sys FFI for IndexAmRoutine with local helper module

## Context

pgrx 0.12 has no `#[pg_index_am]` macro or IndexAmRoutine abstraction.

## Investigation Results

### pg_sys has everything we need

`pgrx::pg_sys` exposes (confirmed in pg17 bindings):

**IndexAmRoutine struct** — all callback function pointers:
```rust
pub struct IndexAmRoutine {
    pub ambuild: ambuild_function,
    pub ambuildempty: ambuildempty_function,
    pub aminsert: aminsert_function,
    pub ambulkdelete: ambulkdelete_function,
    pub amvacuumcleanup: amvacuumcleanup_function,
    pub amcostestimate: amcostestimate_function,
    pub amoptions: amoptions_function,
    pub amvalidate: amvalidate_function,
    pub ambeginscan: ambeginscan_function,
    pub amrescan: amrescan_function,
    pub amgettuple: amgettuple_function,
    pub amendscan: amendscan_function,
    // ... capability flags (amcanorder, amcanorderbyop, etc.)
}
```

**GenericXLog functions**:
```rust
pub fn GenericXLogStart(relation: Relation) -> *mut GenericXLogState;
pub fn GenericXLogRegisterBuffer(state: *mut GenericXLogState, buffer: Buffer, flags: c_int) -> Page;
pub fn GenericXLogFinish(state: *mut GenericXLogState) -> XLogRecPtr;
pub fn GenericXLogAbort(state: *mut GenericXLogState);
```

### Registration pattern

```rust
#[pg_extern]
fn ec_hnsw_handler(_fcinfo: pg_sys::FunctionCallInfo) -> pgrx::PgBox<pg_sys::IndexAmRoutine> {
    let mut amroutine = unsafe { pgrx::PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };
    
    amroutine.amcanorderbyop = true;  // ORDER BY <#>
    amroutine.amcanbackward = false;
    amroutine.amcanunique = false;
    amroutine.amcanmulticol = false;
    amroutine.amsearchnulls = false;
    
    amroutine.ambuild = Some(ec_hnsw_ambuild);
    amroutine.ambuildempty = Some(ec_hnsw_ambuildempty);
    amroutine.aminsert = Some(ec_hnsw_aminsert);
    // ... etc
    
    amroutine
}
```

SQL:
```sql
CREATE ACCESS METHOD ec_hnsw TYPE INDEX HANDLER ec_hnsw_handler;
```

## Decision

**Option A: Raw pg_sys FFI** with a local `src/am/` module for ec_hnsw-specific helpers.

### Module structure

```
src/
├── lib.rs              # pgrx entry, type registration, encode/distance functions
├── am/
│   ├── mod.rs          # ec_hnsw_handler, capability flags
│   ├── build.rs        # ambuild, ambuildempty (uses hnsw_rs for construction)
│   ├── insert.rs       # aminsert (page-level graph update)
│   ├── scan.rs         # ambeginscan, amrescan, amgettuple, amendscan
│   ├── vacuum.rs       # ambulkdelete, amvacuumcleanup
│   ├── cost.rs         # amcostestimate
│   └── page.rs         # Page layout: TqElementTuple, TqNeighborTuple, GenericXLog helpers
├── storage.rs          # TurboCode ↔ bytes serialization for Postgres storage
└── distance.rs         # TurboQuantizer wrapper, Distance impl for hnsw_rs
```

Not building a general AM framework — keeping it specific to ec_hnsw. Every callback is an `unsafe extern "C" fn` that delegates to safe Rust internals where possible.
