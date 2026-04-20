---
id: ADR-033
title: "Shared Graph Lifecycle with Format-Specific Insert/Vacuum Adapters"
status: PROPOSED
impact: Affects FR-010, FR-016, NFR-001, ADR-026, ADR-027, ADR-032
date: 2026-04-16
---
# ADR-033: Shared Graph Lifecycle with Format-Specific Insert/Vacuum Adapters

## Context

ADR-032 commits tqvector to carrying two first-class index formats in `main`:

- **TurboQuant** (current `ScalarV1`)
- **PqFastScan** (current `GroupedV2`)

That raises an architectural question for the remaining task-15 work: should
insert and vacuum stay as one shared implementation, or split into one full
codepath per format?

The current code already shows the tension:

- `src/am/insert.rs` shares one live-insert graph mutation pipeline, but its
  tuple append path is hard-wired to scalar element tuples.
- `src/am/vacuum.rs` shares one graph-repair pipeline, but its load/score path
  is hard-wired to scalar payload decoding.
- `src/am/graph.rs` already has the beginnings of the right boundary:
  `GraphStorageDescriptor` and `GraphTupleRef` distinguish storage formats
  while preserving common graph semantics.

Duplicating all of insert and all of vacuum per format would fork lock-ordering,
duplicate graph mutation logic, and make future correctness fixes land twice.
Trying to make every insert/vacuum detail totally generic would hide real
format differences:

- TurboQuant stores one scoring-hot element tuple.
- PqFastScan stores a hot tuple plus cold rerank tuple and persisted codebooks.
- TurboQuant insert/vacuum score on scalar packed codes.
- PqFastScan insert/vacuum need grouped-search-code and cold-rerank-aware
  storage interpretation.

## Decision

Insert and vacuum will use a **shared graph lifecycle** with
**format-specific adapters** at the payload/storage/scoring seam.

### Shared lifecycle

The following logic remains shared across formats:

- metadata validation and shape checks
- duplicate detection / duplicate heap-TID coalescing
- insert level sampling
- forward-neighbor discovery
- backlink mutation ordering per ADR-026
- vacuum delete-set discovery
- vacuum repair ordering per ADR-027
- metadata drift / entry-point / max-level maintenance

These steps describe graph topology maintenance, not payload representation,
and should not diverge by format.

### Format-specific adapters

The following responsibilities become format-specific:

- encode a new live-insert payload from the input vector
- append and update the persisted tuple set for one graph node
- read an element/hot tuple into a scoring view
- score a candidate element during insert and vacuum repair
- enumerate any extra persisted payloads that vacuum must retain or clean up

The adapter boundary is intentionally narrow. It exists to isolate real storage
differences without forking the whole insert or vacuum algorithm.

### Dispatch shape

Top-level AM entrypoints stay singular:

- one `ec_hnsw_aminsert`
- one `ec_hnsw_ambulkdelete`
- one `ec_hnsw_amvacuumcleanup`

Those entrypoints dispatch early on the metadata-advertised storage format into
format-specific strategy values. The surrounding algorithm remains shared and
invokes adapter hooks where payload/scoring behavior differs.

### What this ADR rejects

#### Full per-format insert/vacuum forks

Rejected because:

- ADR-026 and ADR-027 lock-ordering rules would be duplicated
- graph-topology bug fixes would need to be patched twice
- the common algorithm would become harder to reason about than the current
  single-path implementation

#### Fully generic payload abstraction

Rejected because:

- PqFastScan has real multi-tuple storage that TurboQuant does not
- vacuum needs visibility into hot/cold/codebook persistence, not just a
  shape-erased “element payload”
- overly abstract hooks would hide important format-specific invariants instead
  of documenting them

## Consequences

### Positive

- insert and vacuum keep one authoritative graph-mutation flow
- TurboQuant and PqFastScan can evolve storage independently at the adapter seam
- task 15 can reach parity without entangling lock-ordering and payload-layout
  changes

### Negative

- the first refactor adds indirection before PqFastScan parity is complete
- some current scalar helpers must be reshaped before grouped support can plug in

### Neutral

- this ADR does not by itself implement PqFastScan insert or vacuum parity
- the adapter boundary is internal Rust structure, not a user-visible API

## References

- ADR-026: Live Insert Backlink Lock Ordering
- ADR-027: Vacuum Graph Repair Lock Ordering
- ADR-032: Coexisting Index Formats
- Task 15: Land PqFastScan as First-Class Index Format
