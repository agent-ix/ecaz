---
id: ADR-043
title: "`ecvector` Raw-f32 Column Type"
status: ACCEPTED
impact: Affects FR-001 (type surface), FR-008 (index build), ADR-032, ADR-033, ADR-041
date: 2026-04-18
---
# ADR-043: `ecvector` Raw-f32 Column Type

## Context

The canonical row type is **`ecvector(dim)`** — a raw-f32 column
with `dim` enforced via typmod. `ecvector` is the column type users
put in their tables. It is the source for tqhnsw index builds and
for HeapF32 rerank.

Persisted quantized artifacts, when needed for explicit
family-specific tests or tooling, use family-specific sibling types.
The first such sibling is **`tqvector`** — the TurboQuant-family
persisted quantized artifact. Future families should follow the
same pattern (pick a family-specific sibling name rather than
reusing a generic quantized type). **`tqvector` is not a canonical
row type; it is an artifact type.**

Two workloads motivate raw-f32 as the canonical column-type layer:

1. **pgvector-shaped users.** Users expect a raw f32 column they
   can insert, query, and `<#>`-order. Their mental model is
   pgvector's `vector` type.
2. **Billion-scale HeapF32 rerank.** PqFastScan's
   `GroupedRerankMode::HeapF32` (and any future AM that reranks
   from heap) needs a raw-f32 source column. Expressing that as a
   typed column instead of an opaque `bytea` or a 24-byte-per-row
   `real[]` is the right surface at billion scale.

Quantization remains an index-level concern — the index stores
quantized codes as before. This ADR is strictly about the
canonical heap column type and the row-model correction that
goes with it.

Current head also still carries some explicit `source real[]` /
`rerank_source_column` plumbing in test and benchmark fixtures
where the harness needs a fp32 truth column or an alternate
heap source for a non-default experiment. That harness plumbing
is no longer the canonical product model.

## Empirical motivation

Packet `441` (2026-04-18) measured inline raw-f32 heap storage on
the 50k warm real lane using a `bytea` column with `ALTER COLUMN …
SET STORAGE PLAIN`:

- serious-lane q200 latency: `4.838ms → 3.137ms` (**−35.16%**)
- heap-rerank `decode` bucket: `1386us → 1us`
- recall preserved bit-identical (`graph_recall_at_10 = 0.9629`,
  `mean_abs_score_error = 0`)

That result established that the remaining serious-lane cost on
tqhnsw is **heap-source layout**, not scorer math. The measurement
used the `bytea`+`STORAGE PLAIN` recipe as a research surface.

Current head lands the **type and row-model plumbing** that makes
that optimization meaningful on the real product surface:

- packet `442`: canonical `ecvector` row model
- packet `443`: `tqvector` narrowed to the TurboQuant-family
  sibling artifact type

Important correction: current head does **not** yet guarantee the
packet-`441` win automatically for every `ecvector` column.
`sql/bootstrap.sql` currently declares `ecvector` with
`STORAGE = external`, so the compact raw-f32 datum is landed, but
the storage-policy default that would reproduce the inline hot-path
result without extra tuning remains a follow-up.

Packet `447` (2026-04-19) then measured the actual inline-storage
tradeoff on the canonical `ecvector` surface:

- serious-lane read latency stayed much better inline
  (`5.248ms -> 3.195ms` on the confirming TurboQuant q200 rerun
  from packet `446`)
- total heap+TOAST bytes stayed in the same class
  (`823.0MB` default vs `819.2MB` inline)
- vacuum scan cost stayed effectively flat
  (`19.121s` default vs `19.250s` inline on the 50k seam)
- fresh TurboQuant build time was slightly better inline
  (`180.774s -> 173.784s`, `-3.87%`)
- small row rewrites became materially heavier inline
  (`4.0MB -> 14.3MB` WAL on the steady 1k-row update batch, `3.56x`,
  with HOT dropping from `38` to `0`)

That packet changed the policy readout from "maybe inline should just
be the default" to "inline is a real workload/storage-policy choice":
strong for read-mostly rows, bad for churn-heavy rows.

## Per-row overhead comparison

At 1B rows / 1536-dim, per-row header:

| Heap column | Header | 1B-row header total |
|-------------|--------|---------------------|
| `real[]` | 24 B | 24 GB |
| pgvector `vector` | 8 B | 8 GB |
| `bytea` | 4 B | 4 GB |
| **`ecvector(1536)`** | **4 B** | **4 GB** |

Same wire footprint as `bytea`, 4 B smaller per row than pgvector,
20 B smaller per row than `real[]`. Meaningful at smaller dims and
at billion-row scale.

## Decision

Ship **`ecvector(dim)`** as the canonical row type. Wire layout is
`varlena + raw f32 data` with `dim` enforced via typmod. tqhnsw
can read f32s directly from an indexed `ecvector` column with zero
source reloptions on the native path.

### Wire format

```
+------------------------+
| varlena header (4 B)   |
+------------------------+
| f32[0], f32[1], ...    |   4 * dim bytes
+------------------------+
```

No per-row dim. Typmod stored once in `pg_attribute` is the
authoritative dim; I/O functions validate length against it.

### Typmod contract

- `ecvector(N)` — column is fixed-dim `N`. I/O rejects inserts
  where `length != 4 * N`.
- `ecvector` (no typmod) — column accepts any valid f32 stream
  (`length % 4 == 0`). Current head also allows indexing a
  typmod-less `ecvector` column; dimension consistency is then
  enforced by the build / insert / scan paths as they consume
  row and query values.
- `ALTER TABLE ... ALTER COLUMN v TYPE ecvector(M)` requires a
  table rewrite, matching Postgres' existing behavior for typmod
  changes that narrow the domain.

### Type operations

| Operation | Surface |
|-----------|---------|
| Text I/O | `'[0.1, 0.2, ...]'::ecvector(1536)` |
| Binary I/O | raw f32 stream, length-validated |
| Cast from `real[]` | implicit (assignment) or explicit; length-validated |
| Cast from `float4[]` | same |
| Cast to `real[]` | explicit, materializes an array |
| Cast from pgvector `vector` | deferred; not implemented on current head |
| Operators | `<#>` inner product; `<->` L2 and `<=>` cosine deferred per ADR-032 operator posture |

### tqhnsw integration — canonical path vs optional hooks

`CREATE INDEX ON t USING tqhnsw (v)` on an `ecvector` column works
with zero reloptions:

- BUILD reads f32s directly from the column.
- HeapF32 rerank reads the same column.
- The op class on `tqhnsw` for `ecvector` is `ecvector_ip_ops`.

Current-head nuance:

- `build_source_column` and `rerank_source_column` still exist as
  optional non-default hooks for alternate-source experiments and
  some recall/benchmark fixtures.
- `pq_fastscan` defaults to `heap_f32` when the indexed column is
  `ecvector`.
- `turboquant` still defaults to quantized rerank unless an
  explicit heap-f32 path is selected.

So the canonical row model is landed, but some surrounding helper
surfaces are still broader than the intended end-state.

### Storage policy — deferred to ADR-044

The type decision and the storage-policy decision are separate. The
storage-policy decision (where raw-f32 rerank source lives and what
the default `attstorage` for `ecvector` should be) is tracked in
**ADR-044 — `ecvector` Rerank-Source Location and Storage Policy**.

Why this ADR does not pick a policy:

- Current head declares `ecvector` with `STORAGE = external`, and a pg17
  scratch probe confirmed the storage-code mapping is:
  - `EXTERNAL` => `attstorage = 'e'`
  - `EXTENDED` => `attstorage = 'x'`
  - `MAIN` => `attstorage = 'm'`
  - `PLAIN` => `attstorage = 'p'`
- Packets `441` / `446` therefore measured `EXTERNAL` (default on current
  head) vs `PLAIN`.
- Packet `447` measured the write-path tradeoff of `PLAIN`.
- `EXTENDED` (TOASTed and compressible), `MAIN`, and `PLAIN +
  fillfactor` sweeps are **not yet measured**. The project's M.O.
  is "prove with empirical data" — picking a per-workload default
  off two measured cells would be inference, not measurement.

ADR-044 enumerates the full option space (heap storage modes,
`fillfactor`/structural mitigations, and the architectural
alternative of putting the rerank-source payload in the index
rather than the heap) and defines the measurement plan and
decision criteria. Until ADR-044's matrix lands, no default
change is made; `ecvector` keeps the current-head `EXTERNAL`
default and users with read-mostly workloads can explicitly
choose `PLAIN` as an expert lever while accepting the
packet-`447` write-path tradeoff.

## What landed vs. what remains

### Landed on current head

- `ecvector` type with text/binary I/O
- typmod parsing and enforcement
- casts:
  - `real[] <-> ecvector`
  - `bytea <-> ecvector`
- `ecvector_ip_ops` on `tqhnsw`
- default indexed-column resolution to `ecvector` across build,
  insert, scan, and vacuum
- `tqvector` retained as the TurboQuant-family sibling artifact
  type rather than the canonical row type
- compact canonical `tqvector` artifact layout:
  - per-datum bytes are `dim + gamma + code bytes`
  - `bits=4` and `seed=42` are enforced invariants, not per-row bytes
  - the sibling stays self-describing enough for SQL/operator use
    without carrying the old 8-byte seed field in every row

### Still follow-up work

- storage-policy support that makes the packet-`441` inline/raw
  win a first-class supported path, with explicit guidance about when
  inline storage is and is not appropriate
- cleanup of harnesses and loaders that still stage explicit
  `source real[]` truth columns
- optional pgvector interop casts if we decide they are worth the
  dependency/surface cost

### Quantized sibling artifacts

Persisted quantized artifacts — used for explicit family-specific
tests, tooling, and debugging, not as user-facing row types — live in separate
sibling types. Current head:

- `tqvector` — TurboQuant-family persisted quantized artifact.
  Op class: `tqvector_ip_ops`. Encoder: `encode_to_tqvector(...)`.
  Current wire contract is compact and canonical:
  - row bytes contain `dim`, `gamma`, and packed code bytes
  - `bits` is fixed to `4`
  - `seed` is fixed to `42`
  - text/binary input rejects non-canonical `bits`/`seed`

Rules:

- Sibling types are **never** the default for a user's row column.
  `ecvector` is. A sibling is used only when a test or tool needs
  to materialize the quantized bytes of a specific family for
  inspection / debugging / regression / explicit-compare.
- Adding a new quantized family adds a new family-specific sibling
  (e.g. `pqfsvector` for a PqFastScan-family artifact). Do not
  reuse a generic quantized type name across families.
- The indexed-column resolution layer (`src/am/source.rs`)
  distinguishes canonical `ecvector` from artifact sibling types
  so build / insert / scan / vacuum can treat them differently.
- Current head intentionally keeps `dim` inline in `tqvector` instead
  of hoisting it fully to typmod. The reason is practical, not
  conceptual: PostgreSQL output/operator functions for `tqvector`
  do not receive typmod, so a pure `gamma + code bytes` datum would
  break the current sibling SQL surface. The compact inline-`dim`
  contract is the chosen compromise.

## Scope

- `ecvector` type (catalog, I/O, typmod, casts).
- `<#>` operator and `ecvector_ip_ops` op class on `tqhnsw`.
- tqhnsw build / insert / scan / rerank reading `ecvector` as
  the native source.
- Sibling artifact type contract (family-specific names,
  never canonical).
- Documentation.

Out of scope:

- L2 / cosine operators. Deferred per ADR-032.
- Automatic index rebuild on typmod change. Postgres handles
  this via existing ALTER TABLE rewrite rules.
- Default inline-storage policy or packet-`441` parity by storage
  declaration alone. Current head still uses `STORAGE = external`;
  the storage-policy optimization is a follow-up.
- New quantized sibling types beyond `tqvector`. They follow the
  pattern described above when added.

## Consequences

### Positive

- pgvector-style user experience: `CREATE TABLE` → `INSERT` →
  `CREATE INDEX` → query, no reloptions.
- HeapF32 rerank is a native integration rather than a
  `build_source_column` reloption hack.
- 4 B/row smaller than pgvector's `vector` at billion scale;
  20 B/row smaller than `real[]`.
- Removes the old duplicated-row model and makes packet-`441`'s
  storage-layout win a property we can now pursue on the canonical
  type instead of on a throwaway `bytea` experiment surface.
- Family-specific sibling-type contract keeps the user-facing
  surface one type (`ecvector`) while still allowing explicit
  quantized-artifact tests where needed.

### Negative

- Loss of row-level self-description. A bare `ecvector` datum
  cannot recover its dim without catalog context. Same tradeoff
  pgvector made; not a real problem there.
- Two surface types (`ecvector` canonical + at least one sibling
  artifact) — mitigated by the contract that siblings are
  non-canonical. Docs lead with `ecvector` only; sibling types
  appear only in family-specific test/tooling contexts.
- Current head does not yet make the packet-`441` inline-storage
  result automatic. The type lands the right packed-f32 datum, but
  the heap/TOAST policy that reproduces the large serious-lane win
  still needs an explicit supported surface, and packet `447` shows
  that the choice should be policy-driven rather than universal.

### Neutral

- No change to index on-disk format. `ecvector` is a heap-column
  concern; the index continues to store quantized codes.

## Relationship to other ADRs

- **ADR-032, ADR-033.** The two-format index decision is
  orthogonal to the heap column type. An `ecvector` column feeds
  either PqFastScan or TurboQuant index formats.
- **ADR-041.** `ecvector` type I/O lands under the proposed
  `crate::sql` submodule. Coordinate with stage 6 of ADR-041's
  migration.
- **ADR-042 (native HNSW build).** Native build reading directly
  from heap makes `ecvector` a natural source column — no wrapper
  around `build_source_column` plumbing. The two ADRs compose.
- **ADR-034 (DiskANN), ADR-035 (SPANN).** Both benefit from a
  compact, type-safe canonical row column for HeapF32 rerank at
  billion scale.

## Validation

- **Round-trip correctness.** `real[] → ecvector → real[]` and
  `vector → ecvector → vector` (where available) must be bit-exact.
- **Typmod enforcement.** Inserts with `length != 4 * N` rejected
  on an `ecvector(N)` column.
- **Index parity.** A tqhnsw index built on an `ecvector(1536)`
  column produces identical recall to an equivalent `real[]`
  source baseline.
- **pgvector parity.** Deferred. Current head does not yet
  implement pgvector interop casts.
- **Packet-441 parity.** Not yet guaranteed by current head.
  The type and row-model cleanup are landed, but reproducing the
  `~3.1ms` q200 inline-storage result as a supported default path
  requires follow-up storage-policy work.
- **Sibling-type containment.** No default code path (build /
  insert / scan / vacuum fallback) resolves to a sibling artifact
  type when the indexed column is canonical `ecvector`. Sibling
  types are only reached via explicit configuration or fixture
  setup.

## Open questions

1. **Canonical name.** **RESOLVED** — `ecvector` (Ecaz).
2. **TurboQuant-family sibling name.** **RESOLVED** — `tqvector`
   (TurboQuant family).
3. **Optional pgvector cast — install-time or always?** Still open.
   Current head does not implement pgvector interop casts.
4. **Should `ecvector` without typmod be supported?**
   **IMPLEMENTED** — current head supports typmod-less `ecvector`
   as well as `ecvector(N)`.
5. **Should `ecvector` default to a different storage policy to
   preserve the packet-`441` hot-path result?** Still open, and
   now more precisely framed after packet `447`: the question is
   not just "faster or slower" but whether the product should
   surface inline storage as an explicit per-column mode for
   read-mostly workloads rather than flipping the global default.
