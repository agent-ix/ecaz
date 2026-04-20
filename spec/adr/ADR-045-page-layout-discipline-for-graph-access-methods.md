---
id: ADR-045
title: "Page-Layout Discipline for Graph Access Methods"
status: PROPOSED
impact: Affects ADR-034 (ecdiskann), ADR-041 (multi-AM module structure), ADR-044 (rerank-source location), ADR-046 / ADR-047 (Vamana lock ordering); sets the baseline for AMs added after ecdiskann
date: 2026-04-19
---
# ADR-045: Page-Layout Discipline for Graph Access Methods

## Context

ADR-041 split the cross-AM physical-storage primitives out of
`src/am/page.rs` and into `src/storage/page.rs`. ADR-034 brings the
second access method (`ecdiskann`) onto that shared substrate. ADR-046
and ADR-047 settle the lock-ordering rules for live-insert and vacuum
on the new graph but leave the *tuple shape* and *persistence
sequencing* underspecified.

The current `src/am/diskann/tuple.rs` draft (`VamanaNodeTuple`) is a
straight transcription of `tqhnsw`'s element / neighbor tuple shape. It
inherits three properties that look fine in isolation but become
liabilities once you trace the interactions with `DataPage` /
`DataPageChain`, the metadata page, the rerank-source policy
(ADR-044), and the build-then-persist sequence:

- **Per-tuple storage of index-constant sizes.** The draft repeats
  `graph_degree_r`, `binary_word_count`, and `search_code_len` in
  every tuple even though all three are fixed at index creation and
  already live on the metadata page.
- **Inline `HEAPTID_INLINE_CAPACITY = 10` heap-TID slots in every
  tuple.** That is an HNSW-era HOT-chain accommodation. Vamana
  adjacency is one-node-per-heap-row; nine of those ten slots are
  always zero.
- **Variable tuple length per node.** With `heaptid_count` driving
  layout and the encoded length depending on present-vs-absent fields,
  no two tuples for the same index are guaranteed to be the same
  size, which breaks the cheapest persistence sequencing pattern (see
  §Decision 5).

The cumulative cost at 1536-dim, R=32, grouped-PQ4 is roughly 60 bytes
of dead inline-heaptid padding plus 6 bytes of redundant size fields
*per node*, dropping the tuples-per-page count from ~17 to ~12. That
is a ~40% scan-density regression baked into the tuple format on day
one, before any code reads it.

Beyond the immediate ecdiskann numbers, this ADR is also a deliberate
**baseline-setting exercise**. The project will add more access
methods after ecdiskann (graph variants, IVF families, possibly SPANN
per ADR-035). Each new AM will face the same set of tuple-layout and
persistence-sequencing decisions. Locking poor patterns into ecdiskann
would force every later AM to copy the mistakes (for compatibility) or
re-litigate them (for cleanliness). Settling the discipline once, in
an ADR, lets the next AM start from a known-good baseline.

## Scope

This ADR covers:

- **Tuple-header discipline** for graph-AM node tuples written through
  `storage::page::DataPage` / `DataPageChain`.
- **The contract between metadata page and tuple body** for fields
  that are fixed-per-index.
- **Fixed-per-index tuple length** as a property AMs should preserve
  unless they have an explicit reason to break it.
- **The reusable persistence-sequencing pattern** for AMs that need
  scan-traversal locality.
- **Codec opacity** of tuple bodies relative to the Quantizer /
  QueryScorer trait seam.

This ADR does **not** cover:

- *Where* the rerank source lives (heap vs index cold chain). Owned
  by ADR-044, which is currently deferred behind ADR-042 native build.
  This ADR only reserves the metadata-flag bit and the format-version
  slot needed for ADR-044 to land later without a wire break.
- *Which* encoding to use for search codes (grouped PQ4, RaBitQ,
  scalar-quantized). Owned by ADR-007, ADR-030, ADR-031.
- *Per-node code vs. block-grouped code* layout. Graph-AM specific;
  ecdiskann's choice (per-node, option A) is recorded under
  §Open questions and §Reference layout but is not generalized to
  other AMs by this ADR.
- Lock ordering. ADR-046 / ADR-047 own that.

## Decision

Five rules, applicable to every graph-AM tuple format added from this
point forward unless an explicit deviation is recorded in the AM's
own ADR.

### 1. Per-index-constant fields live on the metadata page, not in tuples

Any field that has the same value for every tuple in a given index
belongs in the metadata page (block 0). Examples that must not be
repeated per tuple in new AMs:

- maximum graph degree (`R`, `M`, equivalent)
- search-code length in bytes
- binary sidecar word count
- code-bits-per-element
- transform / codec kind tags

The decoder reads these from the metadata page once at scan / insert
open time and threads the values into tuple decode calls. This is
already the existing pattern for `tqhnsw`'s `MetadataPage` —
ADR-045 makes it a **rule** for any new AM rather than a convention.

### 2. Tuple bodies are codec-opaque

Tuple-body fields that hold quantized data — search codes, binary
sidecar words, rerank payloads — are stored as **raw byte (or u64)
runs of metadata-declared length**. The tuple format does not
interpret them. Decoding is the Quantizer / QueryScorer trait's job
(per ADR-007 and the trait extraction work in task 17 phase 1).

This keeps the on-disk format stable across codec changes and lets a
single tuple type back multiple codecs differing only in metadata
flags.

### 3. Fixed tuple length per index

Every node tuple in a given graph-AM index encodes to the same byte
length. Length is a function of metadata fields (R, code length,
sidecar width) plus the AM's fixed header.

Variable per-tuple fields (e.g., a `neighbor_count` prefix telling
how many of the R neighbor slots are filled) are allowed *as long as
they do not change the encoded length*. The neighbor slot array is
written at full R width with `ItemPointer::INVALID` in the empty tail.

Unbounded inline collections (e.g., the current 10-slot inline
heaptid array on `VamanaNodeTuple`) are forbidden in the per-tuple
header. AMs that need a variable-length sidecar route the overflow
through a separate chain (`rerank_tid` style), not inline.

This rule is what makes Decision 5 (placeholder-then-patch
persistence) cheap to implement. It also caps the worst-case page
density loss to a known bound — if the AM later wants to vary per
tuple, that is a deliberate format change, not a leak.

### 4. Persistence order is scan-traversal order from the entry point

For graph AMs whose scan starts at a fixed entry point (medoid,
top-layer entry, root), the build-time persistence pass writes node
tuples in BFS-from-entry-point order. Adjacent nodes in the graph
land on adjacent pages, so the scan's early-iteration page reads are
sequential rather than random.

The cost is one extra O(N) BFS pass after build; the benefit is
measurable scan-I/O locality on cold caches. AMs whose scan does not
have a fixed entry point (e.g., partition-style AMs) are exempt and
should record their persistence order in their own ADR.

### 5. Persistence pattern: placeholder-then-patch on `DataPageChain`

For any AM where node N's tuple references node M's TID (which is
itself only known after M is written), persistence runs in two
passes over the same scan-traversal order:

- **Pass 1 — placeholders.** Walk the order. For each node, encode
  a fixed-length tuple with neighbor slots set to
  `ItemPointer::INVALID`, insert it via `DataPageChain::insert_raw_tuple`,
  and record the returned TID into a dense `Vec<ItemPointer>` keyed
  by node id.
- **Pass 2 — patch.** Walk the same order. For each node, re-encode
  the tuple with the resolved neighbor TIDs from the pass-1 map and
  call `DataPageChain::update_raw_tuple` to replace the placeholder
  in place.

This pattern only works when tuple length is fixed-per-index
(Decision 3), which is why those two rules ship together.

To support this without forcing every AM to reinvent the placeholder
pattern, `storage::page` exposes the placeholder-and-patch sequence
through the existing `insert_raw_tuple` + `update_raw_tuple`
primitives — no new shared API is required. AMs that want a thin
helper (`insert_placeholder(payload_len) -> ItemPointer`) may add it
later, but it is not part of the V1 surface.

## Reference layout (ecdiskann-specific, illustrative)

The slim graph-node tuple `ecdiskann` adopts under these rules — to
serve as the worked example for future AMs — is:

```text
[0]  tag: u8                           = TQ_VAMANA_NODE_TAG (0x06)
[1]  flags: u8                         (deleted, has_overflow_heaptids)
[2]  neighbor_count: u16
[4]  primary_heaptid: ItemPointer       (6)
[10] rerank_tid: ItemPointer            (6)   -- INVALID when ADR-044 is in heap-only mode
[16] binary_words: [u64; W]                   -- W from metadata (sidecar width)
     search_code: [u8; C]                     -- C from metadata (grouped-PQ4 length)
     neighbor_slots: [ItemPointer; R]         -- R from metadata; tail = INVALID
```

Header is 16 bytes (vs the draft's 76). All length-determining
fields come from the metadata page. Encoded length is fixed for a
given (R, W, C) triple.

The `rerank_tid` slot is reserved unconditionally even though
ADR-044's current default ("rerank from heap via ecvector EXTERNAL")
leaves it `INVALID`. Reserving the slot in V1 means ADR-044's C1
option (index-side cold-page rerank payload) can land later without
a format break — only the `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` bit on
the metadata page needs to flip.

ecdiskann's per-node-vs-block-grouped code choice (option A,
per-node) is recorded in `plan/design/diskann-build-algorithm.md`
as an ecdiskann-specific decision. This ADR does not generalize it
to other graph AMs.

## Consequences

### Positive

- ecdiskann scan I/O density rises from ~12 to ~17 tuples per 8KB
  page at 1536d / R=32 — a ~40% improvement before any tuning work.
- The rules compose: Decision 3 enables Decision 5, and together
  they make the visit-order persistence (Decision 4) cheap to
  implement in two simple passes.
- Future AMs inherit a known-good starting shape. The
  "should I store this per tuple or in metadata?" question has a
  default answer.
- The metadata-page contract (Decision 1) and codec opacity
  (Decision 2) keep the on-disk format stable as Quantizer
  implementations evolve.
- ADR-044's eventual decision can land without a format break:
  flag bit + slot are reserved.

### Negative

- AMs that actually need variable-length per-tuple state (e.g., true
  HOT-chain heaptid lists, multi-version payloads) must route the
  overflow through a chain rather than inline. This is a small
  amount of extra work versus the inline path.
- The two-pass persistence (Decision 5) doubles the page-write
  count during build. Each page is written once with placeholders
  and then re-written once with patched neighbors. This is bounded
  and only paid at `CREATE INDEX` / native rebuild.
- Decision 4 (BFS-from-entry persistence order) requires holding the
  in-memory graph long enough to do the BFS sweep before any pages
  are flushed. For very large indexes this is the dominant memory
  footprint of build, but it matches what the algorithmic core
  needs anyway (adjacency lists are already in RAM).

### Neutral

- The discipline is enforced by AM authors, not by the type system.
  `storage::page::DataPage` does not refuse a variable-length tuple
  insert. Drift from the rules is caught at code-review time and by
  AM-level tests asserting tuple-length stability.

## Relationship to other ADRs

- **ADR-007 (Query Scoring and Payload).** Decision 2 (codec
  opacity) is the on-disk-format expression of ADR-007's Quantizer
  /  QueryScorer seam. They are the same contract from two sides.
- **ADR-034 (DiskANN Second Access Method).** This ADR sets the
  layout discipline that ecdiskann implements. The Reference
  Layout above is the concrete shape the ADR-034 work uses.
- **ADR-041 (Module Structure for Multi-AM Growth).** This ADR
  formalizes the page-layout contract that Stage-1 of ADR-041
  (the `storage::page` extraction) made physically possible.
  ADR-041 owns the modules; ADR-045 owns the discipline AMs
  follow when they use those modules.
- **ADR-044 (Rerank-Source Location).** Out of scope here. ADR-045
  reserves the `rerank_tid` slot and the
  `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD` metadata bit so ADR-044's
  eventual C1 (index-side cold payload) lands without a format
  break. Until ADR-044 reopens, the slot is unconditionally
  `INVALID` and rerank reads come from the heap `ecvector` row.
- **ADR-046 / ADR-047 (Vamana Lock Ordering).** Decision 5
  (placeholder-then-patch) is build-time only and does not touch
  the live-insert / vacuum lock-ordering protocols. ADR-046's
  insert protocol still applies unchanged once the index exists.
- **ADR-032 / ADR-033 (Coexisting Index Formats).** Decision 1
  (metadata-page contract) and Decision 2 (codec opacity) are
  already what ADR-032 / ADR-033 assume. This ADR makes the
  assumptions explicit.

## Open questions

1. **Per-node code vs. block-grouped code for graph AMs at large
   `R`.** ecdiskann ships with per-node codes (option A). When the
   first graph AM with R ≥ 64 lands, re-evaluate whether
   block-grouped FastScan blocks (option B, the `tqhnsw`
   `TqGroupedHotTuple` shape) become worthwhile. This decision is
   AM-specific and does not retroactively bind ecdiskann.
2. **Should `storage::page` add an `insert_placeholder` helper?**
   V1 leaves AMs to do it manually with `insert_raw_tuple` of a
   zero-filled buffer plus `update_raw_tuple`. If three or more AMs
   end up writing the same boilerplate, lift the helper into
   `storage::page`. Not a blocker now.
3. **How tightly to enforce fixed-tuple-length at the type level.**
   Today the rule is enforced by code-review and by AM-level tests
   asserting `tuple.encoded_len()` is constant. If discipline drifts
   over time, consider a debug-only assertion in
   `DataPageChain::insert_raw_tuple` that all tuples on a chain
   have the same length (with an opt-out for AMs that legitimately
   vary).
4. **BFS-order BUFFER memory at billion scale.** Decision 4 assumes
   the in-memory graph fits in RAM during build. For 1B-scale
   indexes this assumption breaks and a streaming variant is
   needed. ADR-035 (SPANN) territory; not in scope here.
