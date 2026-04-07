---
id: FR-023
title: Strategy Translation Callbacks
type: functional-requirement
status: DRAFT
object_type: entity
traces:
  - US-007
  - FR-009
  - StR-004
---
# FR-023: Strategy Translation Callbacks

## Requirement

On PG18, the extension SHALL implement `amtranslatestrategy` and `amtranslatecmptype` callbacks and set the `amconsistentordering` flag to enable the optimizer to reason about the `<#>` operator's ordering semantics.

Current staged behavior:
- Before the repository has PostgreSQL 18 toolchain support, pure helper-level scaffolding MAY
  encode the intended strategy/CompareType mapping and expose that mapping through read-only
  planner/explain snapshot helpers.
- Those same helpers MAY also model the broader generic `CompareType` domain explicitly so reverse
  mappings to strategy 1 are only accepted for `COMPARE_LT`, while every other compare type falls
  back to `InvalidStrategy` in pure unit-tested code.
- Those scaffolds SHALL report that PG18 strategy-translation callbacks are not yet wired, so
  planner-visible behavior is not implied prematurely.

### CompareType Mapping

tqvector defines one strategy number:

| Strategy | Operator | SQL Usage | CompareType |
|---|---|---|---|
| 1 | `<#>` | `ORDER BY col <#> $q ASC` | `COMPARE_LT` |

The `<#>` operator returns negative inner product (lower = more similar). `ORDER BY ASC` produces results in similarity order. This maps to `COMPARE_LT` semantics: the index returns values in ascending order of the distance metric.

### Callback Implementations

```rust
// amtranslatestrategy: AM strategy → generic CompareType
fn tqhnsw_amtranslatestrategy(strategy: StrategyNumber, _opfamily: Oid) -> CompareType {
    match strategy {
        1 => CompareType::COMPARE_LT,
        _ => CompareType::COMPARE_INVALID,
    }
}

// amtranslatecmptype: generic CompareType → AM strategy
fn tqhnsw_amtranslatecmptype(cmptype: CompareType, _opfamily: Oid) -> StrategyNumber {
    match cmptype {
        CompareType::COMPARE_LT => 1,
        _ => InvalidStrategy,
    }
}
```

### IndexAmRoutine Flags

```rust
amroutine.amconsistentequality = false;   // no equality operator
amroutine.amconsistentordering = true;    // ORDER BY semantics
amroutine.amtranslatestrategy = Some(tqhnsw_amtranslatestrategy);
amroutine.amtranslatecmptype = Some(tqhnsw_amtranslatecmptype);
```

### PG Version Compatibility

On PG17, these fields do not exist in `IndexAmRoutine`. The implementation SHALL use `#[cfg(feature = "pg18")]` guards:

```rust
#[cfg(feature = "pg18")]
{
    amroutine.amconsistentordering = true;
    amroutine.amtranslatestrategy = Some(tqhnsw_amtranslatestrategy);
    amroutine.amtranslatecmptype = Some(tqhnsw_amtranslatecmptype);
}
```

## Acceptance Criteria

### FR-023-AC-1: Strategy translation registered
On PG18, the `IndexAmRoutine` returned by `tqhnsw_handler` SHALL have non-null `amtranslatestrategy` and `amtranslatecmptype` callbacks.

### FR-023-AC-2: COMPARE_LT mapping
`amtranslatestrategy(1, opfamily)` SHALL return `COMPARE_LT`.

### FR-023-AC-3: Reverse mapping
`amtranslatecmptype(COMPARE_LT, opfamily)` SHALL return strategy number 1.

### FR-023-AC-4: Invalid inputs
`amtranslatestrategy(99, opfamily)` SHALL return `COMPARE_INVALID`. `amtranslatecmptype(COMPARE_EQ, opfamily)` SHALL return `InvalidStrategy`.

## References

- PG source: `src/include/access/amapi.h` — `amtranslate_strategy_function`, `amtranslate_cmptype_function` typedefs
- PG source: `src/include/access/cmptype.h` — `CompareType` enum (`COMPARE_INVALID`, `COMPARE_LT`, `COMPARE_LE`, `COMPARE_EQ`, `COMPARE_GE`, `COMPARE_GT`, `COMPARE_NE`, `COMPARE_OVERLAP`, `COMPARE_CONTAINED_BY`)
- PG source: `src/backend/access/index/amapi.c` — `IndexAmTranslateStrategy()` and `IndexAmTranslateCompareType()` wrapper functions that call the AM callbacks
- PG source: `src/backend/access/nbtree/nbtree.c` — `bttranslatestrategy()` and `bttranslatecmptype()` reference implementations
