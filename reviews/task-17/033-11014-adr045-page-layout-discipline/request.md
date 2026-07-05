# Review Request: ADR-045 Page-Layout Discipline for Graph Access Methods

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11001 (task 17 plan), 11002 (ADR-046), 11003 (ADR-047),
1004 (build-algorithm design), 11005 (Phase 1 quantizer trait seam)

## What this packet is

A new ADR (ADR-045, PROPOSED) that fills the page-layout gap left by
ADR-041 stage 1. ADR-041 split cross-AM physical storage primitives
into `src/storage/page.rs`; ADR-045 codifies the **discipline** AMs
follow when they sit on top of those primitives.

The ADR is forward-looking (binds new graph AMs going forward) and
explicitly does **not** retrofit `tqhnsw`. It exists because:

1. `ecdiskann` is the second AM and the first chance to set the
   precedent for a multi-AM future.
2. The current `src/am/diskann/tuple.rs` draft inherits HNSW-era
   layout decisions that cost ~40% scan-page density at 1536d / R=32
   — the cost is paid before any code reads the format.
3. Per a project-level intent recorded with the user: ecdiskann's
   layout choices are deliberately structured as patterns for AMs
   beyond ecdiskann (graph variants, IVF families, possible SPANN per
   ADR-035), not one-off optimizations.

## What changed

### `spec/adr/ADR-045-page-layout-discipline-for-graph-access-methods.md` (new file)

Five rules, applicable to every graph-AM tuple format added from
this point forward unless an explicit deviation is recorded in the
AM's own ADR:

1. **Per-index-constant fields live on the metadata page, not in
   tuples.** Examples: `R`, search-code length, sidecar word count,
   code-bits-per-element, transform/codec kind tags. Decoder reads
   them once from block 0.
2. **Tuple bodies are codec-opaque** — raw byte/u64 runs of
   metadata-declared length. Quantizer / QueryScorer interprets;
   tuple format does not. (On-disk-format expression of ADR-007.)
3. **Fixed tuple length per index.** Variable per-tuple fields
   allowed only if they don't change encoded length (e.g.,
   `neighbor_count` prefix on a fixed-width neighbor slot array).
   Unbounded inline collections forbidden.
4. **Persistence in scan-traversal order from the entry point**
   (BFS-from-medoid for graph AMs with a fixed entry).
5. **Persistence pattern: placeholder-then-patch** on
   `DataPageChain`. Pass 1 inserts fixed-length placeholders with
   `INVALID` neighbors and records each TID; pass 2 re-encodes with
   resolved TIDs and uses `update_raw_tuple`. Works because of
   Decision 3.

Plus a reference layout for `ecdiskann`'s `VamanaNodeTuple` slim
shape (16-byte header, ~660 → ~464 bytes per tuple at 1536d / R=32,
~12 → ~17 tuples per 8KB page).

### What didn't change

- `src/storage/page.rs` — no new shared API. The discipline is *how
  AMs use* `DataPage` / `DataPageChain`, not new primitives on them.
- `src/am/page.rs` (tqhnsw) — explicitly out of scope. tqhnsw's
  V1/V2/V3 wire formats keep their existing tuple shape including
  the 10-slot inline heaptid array. ADR-045 is silent on backporting.
- ADR-044 — owns the rerank-source location decision. ADR-045 only
  reserves the `rerank_tid` slot and `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`
  metadata bit so ADR-044's eventual C1 (index-side cold payload)
  lands later without a wire break.

## Review focus

1. **Is "fixed-per-index tuple length" the right invariant?**
   Decision 3 forbids inline-variable-length state in the per-tuple
   header. AMs that need true variable inline state (e.g., HOT-chain
   versioning à la tqhnsw) must route overflow through a chain. The
   alternative is per-AM length-fields, which break the placeholder-
   patch pattern (Decision 5). Reviewer confirm or push back.
2. **Per-node code vs. block-grouped code is left AM-specific.**
   ecdiskann ships per-node (option A in the §Open questions). The
   ADR does not generalize this — should it? Argument for: it's the
   single biggest scan-density lever for graph AMs, and a
   project-wide rule would prevent re-litigation. Argument against:
   the trade-off is genuinely R-dependent (block grouping wins at
   high R with high in-degree clustering, loses elsewhere). I went
   with against.
3. **Reservation strategy for ADR-044 forward compat.** ADR-045
   reserves the `rerank_tid` slot unconditionally even though the
   current ADR-044 default leaves it `INVALID`. Cost is 6 bytes ×
   N nodes; benefit is no format break when ADR-044 reopens.
   Reviewer confirm this is the right trade.
4. **Enforcement mechanism.** Today the rules are enforced by code-
   review and AM-level tests asserting `tuple.encoded_len()` is
   constant. §Open question 3 floats a debug-only assertion in
   `DataPageChain::insert_raw_tuple` that all tuples on a chain
   have the same length, with an opt-out. Reviewer call: ship that
   guard now or wait for drift to actually happen.
5. **Decision 4 (BFS persistence order) memory cost.** Holds the
   in-memory graph until BFS completes before any pages flush. Fine
   for ≤100M nodes; breaks at 1B-scale. Out-of-scope for ecdiskann
   V1 (10k–10M target) but worth noting for ADR-035 (SPANN).

## Questions to answer

- Is the forward-only scope correct? (Bind new AMs, exempt tqhnsw.)
  The alternative is to also issue a tqhnsw V4 format bump that
  applies the discipline retroactively. I argue no — tqhnsw's V1/V2/V3
  on-disk users exist, the gain is bounded for HNSW (the inline
  heaptid slots are actually used), and the in-flight native-build
  work would collide.
- Should `storage::page` add an `insert_placeholder` helper now or
  wait? §Open question 2. V1 leaves AMs doing `insert_raw_tuple`
  with a zero-filled buffer. If three or more AMs end up writing
  the same boilerplate, lift it; otherwise don't.

## Not doing in this packet

- **Implementation.** This packet is just the ADR. The slim-tuple
  rewrite (Phase 5B) and the placeholder+patch persistence
  (Phase 5C) land separately under packets 11016 and 11009.
- **Backporting to tqhnsw.** Explicitly out of scope per §Scope.
- **Resolving ADR-044.** ADR-044 is deferred behind ADR-042 native
  build per its own decision section. ADR-045 is forward-compatible
  with both possible ADR-044 outcomes.

## Dependencies

- **ADR-007** (Query Scoring and Payload). Decision 2 (codec
  opacity) is the on-disk-format expression of the QueryScorer seam.
- **ADR-034** (DiskANN Second Access Method). The first AM that
  consumes the discipline.
- **ADR-041** (Module Structure for Multi-AM Growth). Stage 1 made
  `storage::page` cross-AM; ADR-045 formalizes the contract for
  AMs sitting on top of it.
- **ADR-044** (Rerank-Source Location). Out of scope here, but
  ADR-045 reserves the slot/flag for ADR-044's eventual C1 option.
- **ADR-046 / ADR-047** (Vamana lock ordering). Untouched. Decision
  5 (placeholder-then-patch) is build-time only; live insert and
  vacuum protocols apply unchanged once the index exists.

## Companion packets

- 11001 — task 17 plan (updated in this slice to reference ADR-045).
- 11002 — ADR-046 draft.
- 11003 — ADR-047 draft.
- 11004 — build-algorithm design doc.
- 11005 — Phase 1 quantizer trait seam.

Future packets in this thread:

- **11015** — Phase 5A landing (in-memory Vamana algorithm core,
  `src/am/diskann/vamana.rs`).
- **11016** — Phase 5B landing (slim-tuple rewrite per ADR-045).
- **11009** — Phase 5C landing (build → persist plumbing, BFS
  persistence, metadata page finalization).

## Definition of ready

- Reviewer accepts or amends the five rules.
- Forward-only scope (no tqhnsw retrofit) confirmed.
- ADR moves from PROPOSED to ACCEPTED, or to ACCEPTED-WITH-
  AMENDMENTS with the deltas listed.
- Phase 5B (slim-tuple rewrite) does not start before this ADR is
  ACCEPTED.

## Handoff notes

This packet is the design layer. The implementation in Phase 5B
(slim tuple) is mechanical given the ADR's reference layout — the
new `VamanaNodeTuple` is a 1:1 transcription of the §Reference
layout block. The implementation in Phase 5C (persistence) is two
loops over a `DataPageChain` plus the metadata-page write; the
algorithmic complexity lives in Phase 5A's `vamana.rs`.

If the reviewer pushes back on Decision 3 (fixed-per-index tuple
length), Decision 5 (placeholder-then-patch) needs revisiting too —
they ship together. The fallback is a single pass with deferred
neighbor patching held in RAM until all TIDs are known, which costs
~6 bytes × N of extra build-time memory but does not require a
fixed-length invariant on the wire format.

If the reviewer accepts the ADR as-is, the implementation order is:

1. Land 11014 (this packet) → ADR-045 ACCEPTED.
2. Land 11015 → `vamana.rs` algorithm core (already drafted, tests
   passing in isolation).
3. Land 11016 → slim-tuple rewrite of `tuple.rs`.
4. Land 11009 → Phase 5C build → persist plumbing.
