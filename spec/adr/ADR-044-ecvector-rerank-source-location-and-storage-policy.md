---
id: ADR-044
title: "`ecvector` Rerank-Source Location and Storage Policy"
status: PROPOSED
impact: Affects ADR-043 (ecvector type), ADR-032 / ADR-033 (two-format index), ADR-042 (native HNSW build)
date: 2026-04-19
---
# ADR-044: `ecvector` Rerank-Source Location and Storage Policy

## Context

ADR-043 landed `ecvector(dim)` as the canonical raw-f32 row type.
Packet `441` identified heap-source storage layout as the dominant
serious-lane cost, and packet `446` confirmed the win survives on
the canonical `ecvector` surface when storage is forced inline
(`-39.12%` TurboQuant q200, `5.248ms → 3.195ms`).

Packet `447` then measured the inline-storage tradeoff on the same
fixture. The write-path penalty is not small:

- **WAL on small row rewrites:** `4.0MB → 14.3MB` per 1k-row
  steady batch (`3.56×`)
- **HOT updates:** `38 → 0` — HOT is lost entirely on inline
- **Heap working set:** `468 → 50,000` pages (`2.86% → 305%` of
  a `128MB shared_buffers`)
- **Build time:** slightly faster inline (`-3.87%`)
- **Vacuum:** essentially flat

ADR-043 has since been extended with `§Storage policy guidance`
that treats inline as a per-column workload-specific mode
(`EXTERNAL` for churn-heavy, `PLAIN` for read-mostly). A pg17
scratch probe then corrected an important detail in that framing:
for this server, `ALTER COLUMN ... SET STORAGE EXTERNAL` produces
`attstorage = 'e'`, while `SET STORAGE EXTENDED` produces
`attstorage = 'x'`. Current head declares `ecvector` with
`STORAGE = external`, so packet `446` / packet `447` already
measured the `EXTERNAL` default surface, not `EXTENDED`.

That makes the remaining decision surface narrower and more precise:
the question is no longer "should `ecvector` default to EXTERNAL or
PLAIN?" It is "is the current `EXTERNAL` default the right stopping
point, or do we need `PLAIN` as an expert lever and/or the
architectural alternative of moving the rerank payload out of the
heap entirely?" This ADR enumerates that full option set so the
decision is informed rather than implicit.

## Scope

This ADR is about **where the raw-f32 rerank source lives**, not
about the type surface. Concretely:

- Which heap storage mode is the right default for `ecvector`?
- Is the heap the right home at all, or should the rerank source
  live in the index?
- What mitigations exist for the write-path cost of inline heap?

ADR-043 keeps ownership of the type definition, typmod contract,
casts, and operator surface. This ADR does not propose changing
any of those.

## Decision

**Not yet made.** Status is PROPOSED. Coder-1 runs the measurement
matrix in §Measurement plan, and the decision criteria in
§Decision criteria select one of the options in §Option catalog.

## Option catalog

Three families: heap-storage tuning, heap-storage + structural
mitigations, and architectural alternatives that move the source
out of the heap entirely.

### Family A — heap-storage modes for `ecvector`

`ecvector` is a varlena. PostgreSQL offers four `attstorage` modes.

#### A1. `EXTERNAL` (default on current head)

TOAST without compression. Keeps small heap tuples (UPDATEs cheap,
HOT viable) while avoiding the compression step; rerank still pays
detoast, but not decompress.

- **Serious-lane latency:** measured baseline on current head
  (`5.248ms` TurboQuant)
- **Small-update WAL:** measured baseline (`4.0MB / 1k batch`)
- **HOT:** viable
- **Heap working set:** smallest measured current-head surface

This is the current default, not a hypothesis.

#### A2. `EXTENDED`

TOAST with compression. Same small-heap-tuple / HOT-viable update
shape as `EXTERNAL`, but adds decompression work on rerank reads in
exchange for a smaller toasted footprint.

- **Serious-lane latency:** **unmeasured** — hypothesis is
  "same class as `EXTERNAL`, but worse"
- **Small-update WAL:** expected similar to `EXTERNAL`
- **HOT:** expected viable
- **Heap working set:** similar heap footprint to `EXTERNAL`
- **TOAST bytes:** expected smaller than `EXTERNAL`

This is now the highest-value unmeasured heap-storage cell, because
it tells us whether the current default should stay uncompressed or
whether compressed TOAST is viable on the serious lane.

#### A3. `MAIN`

Inline where possible, TOAST only when the tuple exceeds the page
limit. For 1536-dim / 6 KB tuples on 8 KB pages, most rows go
inline by default (since a 6 KB tuple fits with headers), so this
is close to `PLAIN` in practice for this dim.

- **Serious-lane latency:** expected similar to `PLAIN`
- **Small-update WAL:** expected similar to `PLAIN`
- **HOT:** expected similar to `PLAIN` (lost)

Probably not interesting as a distinct cell at 1536-dim, but worth
a single-cell sanity check.

#### A4. `PLAIN` (measured in packet `447`)

Forced inline. Best read latency, worst write-path cost.

- **Serious-lane latency:** best (measured: `3.195ms` TurboQuant)
- **Small-update WAL:** `14.3MB / 1k batch` (`3.56×`)
- **HOT:** lost entirely
- **Heap working set:** `305%` of `128MB shared_buffers` on 50k

### Family B — `PLAIN` + structural/mitigation knobs

Assumes the user wants the `PLAIN` read-latency profile but needs
to mitigate the write-path cost.

#### B1. `fillfactor < 100`

HOT updates require the new tuple version fit on the same page as
the old. At `fillfactor = 100` with 6 KB tuples, pages are near-full
after insert and HOT has nowhere to land. `fillfactor = 70-80`
leaves ~1.6-2.4 KB free per page, which may restore HOT for
moderate non-indexed-column churn at the cost of ~20% extra heap
pages.

- **Serious-lane latency:** expected unchanged from `PLAIN`
- **Small-update WAL:** expected reduced if HOT is restored
- **HOT:** hypothesis is "restored for moderate churn"
- **Heap working set:** `+20-30%` vs `PLAIN / fillfactor=100`

Cheap to measure. ADR-043 dismissed this as "not the primary
answer" without data.

#### B2. Structural vertical partitioning

Keep `(id, embedding)` in one table, `(id, metadata…)` in another,
join at query time. User-side mitigation, no extension work. Fully
eliminates the write-path penalty because the embedding row is
never touched by metadata updates.

- **Serious-lane latency:** same as `PLAIN`
- **Small-update WAL:** same as the unrelated metadata table
  (small)
- **HOT:** preserved on the metadata table
- **Cost:** join at query time, application-side complexity

Already the primary guidance in ADR-043's mitigation section.
Worth naming here as the canonical "user-side" answer that
complements any internal choice.

### Family C — architectural alternatives (move source out of heap)

These relocate the raw-f32 rerank source somewhere other than the
base heap row. They cost more engineering but fully decouple heap
row churn from rerank source storage.

#### C1. Index-side cold-page rerank payload

The index tuple already carries quant codes. Add a cold-page
inline-f32 payload (same shape pre-`442` `persisted_source_column`
had, but owned by the index, not a user column). Heap `ecvector`
column stays at `EXTENDED` / `EXTERNAL`; UPDATE path reverts to
small-heap-tuple / HOT-viable behavior because the embedding is
not in the heap row.

- **Serious-lane latency:** expected comparable to `PLAIN` (cold
  page is in `shared_buffers`, same cache class as heap)
- **Small-update WAL:** expected similar to `EXTENDED` (heap row
  is small)
- **HOT:** viable
- **Heap working set:** small (same as `EXTENDED`)
- **Index size:** `+4*dim` bytes per entry (`+6 KB` at 1536-dim)
- **Build cost:** higher (index writes f32 payload during build)
- **Rebuild cost:** higher (every CREATE INDEX re-writes f32)

Engineering cost: index format change (`INDEX_FORMAT_V4` or
equivalent), new cold-page wire layout, build/scan/vacuum paths
updated. Composes with ADR-042 (native HNSW build) — native build
would write the payload directly from the heap `ecvector` column
during index construction.

This is the cleanest architectural answer if `EXTERNAL` (A2) does
not recover enough of the serious-lane win to make A2 the default.

##### Current-code fit for C1

Current head already has most of the seam C1 needs:

- **Separate hot and cold tuple ownership already exists.**
  TurboQuant V3 stores a hot tuple (`TqTurboHotTuple`) that points
  at a separate cold rerank tuple (`TqRerankTuple`) via `reranktid`.
  PqFastScan uses the same pattern with `TqGroupedHotTuple` plus
  `TqRerankTuple`.
- **Build already stages rerank tuples independently.**
  `src/am/build.rs` writes the rerank tuple first, then stages the
  hot tuple with the resulting `reranktid`.
- **Insert already writes the rerank tuple through a dedicated path.**
  `src/am/insert.rs` encodes and writes the rerank payload before the
  hot tuple on live insert.
- **Scan and vacuum already resolve rerank payload through the index.**
  `src/am/graph.rs::load_rerank_payload(...)` and the grouped variant
  already fetch the cold payload by `reranktid`, and vacuum's linear
  repair path already depends on that separation.

So C1 is not "invent a second storage plane from scratch". The
current code already has a cold-payload indirection point. The real
format decision is:

- **Option C1a: widen `TqRerankTuple`.** Add raw-f32 bytes to the
  existing cold tuple. Lowest tuple-count overhead, but it overloads a
  tuple that current scan/vacuum code assumes is just `gamma + code`.
- **Option C1b: add a sibling cold raw-f32 tuple kind.** Keep
  `TqRerankTuple` for `gamma + code`, add a second cold tuple for the
  exact rerank source, and extend the hot tuple / metadata layout with
  a second TID or a payload flag. Slightly more page/metadata work, but
  cleaner for backwards reasoning because current quantized-score paths
  keep their existing tuple contract.

The likely implementation shape is therefore:

1. bump the on-disk format (`INDEX_FORMAT_V4` or equivalent)
2. add a new cold raw-f32 tuple layout or widen the existing cold
   tuple deliberately
3. teach build/insert to materialize that payload from the indexed
   `ecvector` heap datum while writing the index
4. teach scan/vacuum/repair paths to consume the cold raw-f32 payload
   for exact rerank without heap fetches

That is why C1 composes naturally with ADR-042 native HNSW build:
the native build path is the right place to populate the cold payload
once, directly from the canonical `ecvector` column, while the index is
being written anyway.

#### C2. AM-owned sidecar relation

A dedicated relation the AM maintains, holding raw f32 keyed by
heap TID. Separate from both the user heap and the index. The AM
keeps it in sync on INSERT/UPDATE/DELETE/VACUUM. Closest analogue:
how `btree` maintains a separate index relation.

- **Serious-lane latency:** same class as C1 (reads from a
  well-behaved relation)
- **Write path:** additional maintenance write per base-row write
  (AM trigger-like path); engineering cost is the AM plumbing
- **Heap working set:** small (base row is untouched)
- **Storage:** same as C1 but in its own relation

More engineering than C1, not obviously better on any axis
measured here. Named for completeness; unlikely to be the answer
unless C1 conflicts with an index-format constraint we have not
yet hit.

#### C3. Custom TOAST strategy (pg16+)

Postgres 16 supports custom per-column TOAST strategies. Could
store the f32 payload in a dedicated TOAST relation with access
patterns optimized for sequential f32 fetch.

- **Serious-lane latency:** depends on implementation
- **Small-update WAL:** inherits the `EXTENDED` / `EXTERNAL`
  shape (small base row)
- **Postgres version floor:** 16
- **Engineering cost:** high, and with an unknown payoff

Probably not the answer unless we are already committed to a pg16
floor for other reasons.

### Family D — accept the quality tradeoff

#### D1. Quantized rerank only, no raw-f32 rerank

Remove heap-f32 rerank from the default path. Rerank reads quant
codes from the heap (the pre-`442` model) or from the index. This
drops the "exact rerank bits" gain from packet `441` / packet `446`
but eliminates the entire storage/location question.

- **Serious-lane latency:** matches the pre-`441` quantized-only
  profile
- **Small-update WAL:** matches `EXTENDED`
- **HOT:** viable
- **Recall quality:** bounded by quant, not exact

Named for completeness. The whole arc that produced ADR-043 was
motivated by the exact-rerank quality gain, so this option would
be a retreat.

## Measurement plan

Coder-1 runs these cells on the same fixture packets `446` / `447`
used (`task16_ecvector` DB, 50k real corpus, m=16, ef_search=128,
q200, `warm-after-prime3`, `cached-plan` timing, confirming
reruns).

### Must-measure (block the decision until landed)

1. **A2: `EXTENDED` cell.** Same table as `447`'s default surface
   but with `ALTER COLUMN embedding SET STORAGE EXTENDED`.
   Measure:
   - serious-lane q200 latency (TurboQuant and PqFastScan,
     confirming reruns)
   - WAL and HOT on the same 1k-row steady update probe
   - heap + TOAST bytes
   - buffer-cache pressure
   - build time
2. **B1: `PLAIN` + `fillfactor` sweep.** Three cells at
   `fillfactor = 70, 80, 90`. Focus on WAL/HOT on the update
   probe. Serious-lane latency and build time can be spot-checked
   for regressions but should be effectively unchanged.
3. **A3: `MAIN` sanity check.** One cell. Likely close to `PLAIN`
   at 1536-dim; measurement confirms or denies.

### Should-measure (informs the architectural decision)

4. **Decompose packet-`441`'s `1386us` decode bucket into detoast
   vs decompress components.** If decompression is a large fraction,
   A2 (`EXTENDED`) is unlikely to be viable and A1 (`EXTERNAL`)
   remains the only heap-default candidate. If detoast dominates,
   neither `EXTERNAL` nor `EXTENDED` closes much of the gap to
   `PLAIN`, which strengthens the case for the architectural option
   (C1).
5. **Update probe with a larger touched column.** Packet `447`'s
   probe touched a 4-byte `integer`. A cell that touches a larger
   non-embedding column (e.g., a 100-byte text) tests whether the
   `3.56×` WAL multiplier is worst-case-only or applies broadly.

### Estimate (informs whether to build, not measure)

6. **Engineering sketch for C1 (index-side cold-page payload).**
   Not a measurement — a design note. Should cover:
   - index format change (wire, versioning, rebuild story)
   - build-path integration (composes with ADR-042 native HNSW?)
   - scan-path cold-page fetch
   - vacuum behavior
   - rough bytes-per-entry and build-time hit

## Decision criteria

Once the measurement plan lands, the decision selects **one**
option as the default product storage for `ecvector`, plus any
number of additional options as documented expert knobs.

### Selection rules

- **If A2 (`EXTENDED`) stays within a small latency tax of
  A1 (`EXTERNAL`) while materially reducing TOAST bytes:** keep A1
  (`EXTERNAL`) as the default, and document A2 (`EXTENDED`) as the
  space-first expert knob. C1 is tabled unless the A1→A4 gap is still
  large enough to justify it on product grounds.
- **If A2 regresses sharply versus A1 while A1 still sits far from
  A4 (`PLAIN`):** A1 remains the default and C1 moves to a funded
  architectural track. The heap default is a stopgap while C1 lands.
- **If A2 regresses sharply and C1 is not yet funded:** A1
  (`EXTERNAL`) remains the default and `PLAIN` stays the documented
  expert knob until C1 ships.
- **If B1 (`fillfactor`) restores HOT cleanly on the update
  probe at <25% heap overhead:** document `PLAIN + fillfactor=80`
  as an expert pattern in ADR-043's mitigation section, regardless
  of which default wins. Does not affect the default choice
  itself.

### Non-criteria

- "Which is faster in the best case" is not enough — the write-
  path cost is the load-bearing question.
- "Which gives the biggest serious-lane win" is not enough —
  the product default has to be safe for churn-heavy workloads
  as well as read-mostly ones.

## Consequences

### Positive (once decided)

- A single defensible default storage mode for `ecvector`,
  measured not guessed.
- Expert knobs (PLAIN, fillfactor tuning) explicitly documented
  with their tradeoff shapes instead of discovered by users in
  production.
- C1 either lands as a future architectural track or is closed
  with a clear reason.

### Negative

- The measurement plan is several cells and takes non-trivial
  scratch-DB time. Blocks task-16 merge-ready on this ADR's
  resolution.
- If C1 lands, it implies an `INDEX_FORMAT` bump with all the
  rebuild/compat implications that carries.

### Neutral

- ADR-043's type decision is unaffected. `ecvector(dim)` is still
  the canonical row type regardless of which option wins here.

## Relationship to other ADRs

- **ADR-043.** Owns the type. This ADR owns the storage/location.
  ADR-043's current `§Storage policy guidance` section will be
  trimmed to reference this ADR once the decision lands.
- **ADR-032 / ADR-033.** The two-format index decision is
  orthogonal. Either index format can read rerank source from any
  of the options in §Option catalog.
- **ADR-042 (native HNSW build).** Option C1 composes cleanly
  with native build — native build is the natural place to write
  an index-side cold-page f32 payload. If C1 is chosen, this ADR
  and ADR-042 should be implemented together, not serially.
- **ADR-031 (RaBitQ sidecar).** Unaffected. The sign-bit sidecar
  is a scoring-stage artifact, not a rerank source.

## Open questions

1. **How much extra serious-lane tax does `EXTENDED` add over the
   current `EXTERNAL` default?** Must-measure cell 1.
2. **Does `EXTENDED` buy enough TOAST-footprint reduction to matter
   at the 50k seam?** Same cell; this is the only reason to consider
   it over the current default.
3. **What is the engineering cost of C1 (index-side cold-page
   payload)?** Should-estimate item 6. Drives whether C1 is in
   scope as a product answer or a deferred follow-up.
4. **Does the `3.56×` WAL multiplier hold for updates that touch
   larger non-embedding columns?** Should-measure cell 5. Shifts
   the framing from "inline is dangerous for any churn" to "inline
   is dangerous for tiny-touch churn specifically."
5. **Does `fillfactor` actually restore HOT on this workload?**
   Packet `447` conjectured no-without-measurement. Cell B1
   closes this.
