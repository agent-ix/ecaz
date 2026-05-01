---
id: ADR-041
title: "Module Structure for Multi-AM, Multi-Quantizer Growth"
status: IMPLEMENTED
impact: Affects ADR-032, ADR-033, ADR-034, ADR-035, ADR-036, ADR-037, ADR-038, ADR-039, ADR-040, ADR-048
date: 2026-04-18
---
# ADR-041: Module Structure for Multi-AM, Multi-Quantizer Growth

## Context

Through ADR-032 and ADR-033, tqvector committed to carrying two first-class
quantizer formats (TurboQuant, PqFastScan) behind a single access method
(`ec_hnsw`). The roadmap queued up by ADR-034–ADR-040 substantially widens
that surface:

- **Multiple access methods** on the scale ladder: `ec_hnsw` (current),
  `ec_diskann` (ADR-034), `ec_ivf` (ADR-048), and future algorithms. They
  share quantizer/scoring kernels where practical; they differ in graph,
  posting-list, or other outer index structure and on-disk layout.
- **More quantizer families.** OPQ (ADR-036) extends PqFastScan with a
  learned rotation; AQ/RVQ (ADR-037) is a structurally new family with its
  own encoding and scoring kernel; LSQ (ADR-038) is a codebook-refinement
  trick.
- **More SIMD backends.** AVX-512 and ARM SVE/SVE2 (ADR-039) join AVX2 and
  NEON under runtime dispatch.
- **New cross-AM coordination primitives.** Parallel index scan (ADR-040)
  introduces a shared coordinator that every AM should inherit.

Today's tree assumes a single AM, with two quantizer formats threaded
through that AM's code via `GraphStorageDescriptor` match sites. Concrete
pressure points:

- `src/am/` is structured for one AM. All names and constants are
  `EC_HNSW_*`. `routine.rs` builds one `IndexAmRoutine`. A second AM has
  no natural home.
- `src/am/scan.rs` (6,685 lines) multiplexes quantizer format through
  seven `match GraphStorageDescriptor` sites plus a set of
  `PqFastScan*ModeDecision` enums. A third quantizer family would require
  threading another arm through every site.
- `src/quant/prod.rs` (2,135 lines) is a single `ProdQuantizer` type that
  encodes both families' state. TurboQuant-shaped internals (MSE + QJL)
  with grouped-PQ bolted alongside. AQ/RVQ will not fit this shape.
- `src/am/options.rs` defines `StorageFormat` inside ec_hnsw-scoped
  options. ec_diskann cannot reach it without duplication.
- `src/am/page.rs` (2,209 lines) mixes ec_hnsw-specific tuple layouts
  with genuinely cross-AM primitives (`ItemPointer`, page chaining, WAL
  wrappers).
- `src/quant/simd.rs` is a single dispatch file. Two more backends put
  four distinct implementations per hot kernel behind one file.
- `src/lib.rs` has grown to 19,590 lines holding PG init, the SQL datum
  surface, cross-module helpers, `bench_api` re-exports, and inline unit
  tests.

Left alone, each ADR in the 034–040 sequence pays an integration tax and
each tax compounds the next. The cost of reorganizing goes up the longer
we defer.

## Decision

Reshape `src/` around three orthogonal axes (access method, quantizer
family, SIMD backend), land three load-bearing trait seams, and stage the
migration against existing task boundaries rather than as a flag-day
rewrite.

### Three axes, three module boundaries

```
src/
├── lib.rs                      slim — pgrx init, re-exports only
├── sql/                        SQL function surface, grouped by purpose
│   ├── datum.rs                tqvector in/out/recv/send
│   ├── scoring.rs              tqvector_inner_product, etc.
│   └── snapshots.rs            *_snapshot introspection
│
├── storage/                    cross-AM physical storage primitives
│   ├── page.rs                 PageHeader, DataPage, ItemPointer
│   ├── wal.rs                  GenericXLogTxn wrapper
│   └── metadata.rs             shared metadata-page framework
│
├── quant/
│   ├── mod.rs                  Quantizer + PreparedQuery traits
│   ├── simd/                   per-backend kernel files
│   │   ├── scalar.rs  avx2.rs  avx512.rs  neon.rs  sve.rs
│   ├── common/                 shared primitives
│   │   ├── hadamard.rs  rotation.rs  opq.rs  codebook.rs  lsq.rs
│   ├── turboquant/             self-contained family
│   ├── pqfastscan/             self-contained family
│   └── aq/  rvq/               future families, same shape
│
├── am/
│   ├── common/                 cross-AM: cost, explain, stats, stream,
│   │                           parallel, reloption parsing
│   ├── ec_hnsw/                 build, scan, insert, vacuum, graph,
│   │                           search, page, options, routine
│   ├── ec_diskann/              sibling of ec_hnsw
│   └── ec_ivf/                 IVF posting-list AM
│
└── bin/
```

### Three load-bearing seams

**`Quantizer` trait** (in `crate::quant`) — one impl per family. Owns
training, encode, query preparation, wire-format version. Scan code holds
a `&dyn Quantizer` and a `&dyn PreparedQuery`; it does not know which
family it holds.

```rust
pub trait Quantizer: Send + Sync {
    type Prepared: PreparedQuery;
    fn encode(&self, v: &[f32]) -> Box<[u8]>;
    fn prepare(&self, q: &[f32]) -> Self::Prepared;
    fn code_len(&self) -> usize;
    fn wire_format_version(&self) -> u32;
}

pub trait PreparedQuery {
    fn score(&self, code: &[u8]) -> f32;
}
```

**`TupleCodec` trait** (per AM) — reads and writes tuple headers (level,
neighbors, heap TIDs) while treating quantized bytes as an opaque `&[u8]`.
Decouples graph tuple layout from quantizer choice inside an AM.

**`ParallelScanCoordinator`** (in `am/common/parallel.rs`) — AM-agnostic.
Holds the shared top-K heap and DSM layout described by ADR-040. Each AM
calls into it from its scan path; no AM owns a copy.

### Consequence for `StorageFormat`

The enum moves out of `am/ec_hnsw/options.rs` and becomes a crate-level
`quant::Family` enum. Any AM can reference it; each AM carries its own
reloption that resolves to the shared enum. This lets ec_diskann adopt
PqFastScan (and future families) without a parallel type.

## Staged migration

The reshape is paired with tasks that already need the change. No
flag-day rewrite.

| Stage | Paired task | What lands | Cost |
|-------|-------------|------------|------|
| 0 | before task 17 merges | Extract `Quantizer` and `PreparedQuery` traits. TurboQuant and PqFastScan both implement them. Scan call sites route through the trait. **No file moves.** | small |
| 1 | start of task 17 (DiskANN) | Move `page.rs`, `wal.rs`, `ItemPointer` into `crate::storage::*`. Keep AM-specific tuple codec logic where it is. | medium (import churn) |
| 2 | task 17, first PR | Rename `am/*` contents to `am/ec_hnsw/*`. Extract `am/common/` (cost, explain, stats, stream, planner, parallel). Float `StorageFormat` into `crate::quant::Family`. | medium, atomic |
| 3 | task 17, second PR | Add `am/ec_diskann/` as a peer module. Register second `IndexAmRoutine`. | core of task 17 |
| 4 | start of task 21 (SIMD) | Split `quant/simd.rs` into per-backend files under `quant/simd/`. | small |
| 5 | task 22, if reached | Split `quant/prod.rs` into per-family modules under `quant/turboquant/` and `quant/pqfastscan/`. AQ/RVQ forces this; OPQ does not. | medium |
| 6 | opportunistic | Slim `lib.rs` into `sql/` submodules as surfaces accrete. | incremental |

### Task 17 takes ownership of stages 0–3

Task 17 (DiskANN, ADR-034) is the first task that genuinely requires two
coexisting access methods. It rolls in:

- Stage 0 trait extraction as preparation.
- Stage 1 storage-primitive move as its first PR.
- Stage 2 `am/` reshape as its second PR.
- Stage 3 `am/ec_diskann/` as the rest of the task.

Subsequent tasks (18 parallel scan, 20 OPQ, 21 SIMD, 22 AQ/RVQ) inherit
the new structure and only pay for their own incremental work.

### What is explicitly NOT in scope of this ADR

- **`ScoringStrategy` trait inside `scan.rs`** (advisory note on task 15).
  Keep the inline `match GraphStorageDescriptor` shape until AQ/RVQ
  (task 22) forces the issue. OPQ (task 20) is still PqFastScan with a
  different transform; it does not justify the trait.
- **Multi-operator support** (cosine, L2). Only if a real user asks.
  Orthogonal to structure.
- **Parallel build.** Separate scope from parallel scan (ADR-040).

## Consequences

### Positive

- Task 17 lands DiskANN as a peer access method, not a fork of ec_hnsw.
- Task 18 writes its parallel-scan coordinator once, in `am/common/`,
  and all AMs inherit it.
- Task 20 adds OPQ as a `Quantizer` impl (or a transform variant inside
  the pqfastscan family) without touching scan logic.
- Task 21 adds AVX-512 and SVE as new files under `quant/simd/` without
  editing one large dispatcher.
- Task 22 (if reached) introduces AQ/RVQ as a new `quant/aq/` or
  `quant/rvq/` module implementing the same `Quantizer` trait. No
  surgery on PqFastScan.
- Cross-AM reloption sharing falls out of stage 2 for free.

### Negative

- Stage 2 touches every import site in `am/` and `lib.rs`. Must land as
  one atomic PR; partial state is ugly.
- The staged plan concentrates restructuring work on the task-17 owner.
  Trading that concentration for a cleaner result elsewhere is the whole
  bet. Task 17's schedule absorbs the cost.
- Trait indirection costs a small amount of inlining visibility. For the
  scoring hot path, profile-guided inlining or `#[inline]` hints on the
  `PreparedQuery::score` impls should keep this invisible; validate
  against task-08 benchmarks before and after stage 0.
- Rust's compile-time dispatch via generics (rather than `&dyn`) would
  eliminate the inlining question entirely but would force `scan.rs` to
  be generic over `<Q: Quantizer>`. Revisit if the virtual call shows up
  in profiles; otherwise prefer `&dyn` for simpler type plumbing.

### Neutral

- Tests move with their modules. No test consolidation is implied.
- `bench_api` re-exports in `lib.rs` are preserved as a stable surface
  for external callers; the re-exports just point to new module paths.

## Validation

- **Before stage 0:** record current hot-path benchmark numbers from
  task 08 (`prepare_ip_query/d1536_b4`, `score_ip_encoded/d1536_b4`).
- **After stage 0:** the trait-indirected scoring path must match those
  numbers within noise (±5%). If it regresses meaningfully, pivot to
  generics instead of `&dyn`.
- **After stage 2:** the 50k warm real seam recall and latency results
  must match pre-reshape numbers byte for byte (recall) or within noise
  (latency). This is an equivalence check, not a performance run.

## Relationship to other ADRs

- **ADR-032, ADR-033** — the current two-format decision. This ADR
  provides the scaffolding that keeps those decisions tractable as the
  format count grows.
- **ADR-034 (DiskANN)** — the forcing function for stages 1–3. Adopts
  this ADR's module shape as a prerequisite.
- **ADR-035 (dropped), ADR-036 (OPQ), ADR-037 (AQ/RVQ), ADR-038 (LSQ),
  ADR-048 (IVF)** — consumers of the shape this ADR lands.
- **ADR-039 (ARM SVE)** — stage 4 is the home for its new backend files.
- **ADR-040 (parallel scan)** — the coordinator this ADR names lives
  in `am/common/parallel.rs`.

## Open questions

1. Should `am/common/` also own the `IndexAmRoutine` vtable *builder*
   (a helper that each AM calls), or does each AM construct its own?
   Lean: each AM constructs its own; the callback choices differ enough
   that a shared builder has low payoff.
2. Where do custom pgstat-kind registrations live once we have three
   AMs? Tentative: one shared `tqvector_stats` kind with per-AM
   counter blocks. Revisit during task 19 (PG18 completion) when the
   registration lands.
3. SQL surface grouping under `sql/` — by theme (datum, scoring,
   snapshots) or by visibility (public API, dev-only)? Defer until
   stage 6 actually runs.
