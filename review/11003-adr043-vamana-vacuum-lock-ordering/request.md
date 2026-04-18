# Review Request: ADR-043 Vamana Vacuum Graph Repair Lock Ordering

Branch: `adr034-diskann-access-method`

Scope:
- `spec/adr/ADR-043-vamana-vacuum-lock-ordering.md`

## What this slice is

Draft ADR-043 (status: PROPOSED) covering the lock-ordering
protocol for vacuum-time graph repair on the upcoming
`tqdiskann` access method. Analog of ADR-027 for HNSW, adapted
to single-layer Vamana, fixed-`R` neighbor lists, and
α-aware fill.

No code lands under this slice. ADR-043 must move to ACCEPTED
before task 17 phase 5 (vacuum implementation) begins.

## What changed

New file: `spec/adr/ADR-043-vamana-vacuum-lock-ordering.md`.
Status PROPOSED.

Decision encodes ten rules:

1. Pass 1 dead-heap-TID discovery (hot chain plus cold chain).
2. Pass 2 neighbor-array repair scan in ascending block order.
3. Repair planning is read-only before any exclusive lock.
4. Reopen one page under `BUFFER_LOCK_EXCLUSIVE`; never upgrade
   in place.
5. Fill-only writes under the exclusive lock; no live-neighbor
   eviction.
6. One data-page `EXCLUSIVE` at a time, ascending block order;
   hot-then-cold for same element.
7. No metadata-page `EXCLUSIVE` during pass 2.
8. Replan-before-retry across ordered passes.
9. Pass 3 finalization flips `deleted = true` on orphans.
10. Entry-point medoid repair deferred: set
    `needs_medoid_refresh` metadata flag, migrate at rebuild.

Delta from ADR-027:

- Single-layer, flat neighbor list (no layer-segmented tuple).
- α-aware candidate scoring during fill.
- Explicit cold-rerank-payload-chain ordering.
- Entry-point-medoid deferral (step 10).

## Review focus

- **Fill-only posture (step 5).** ADR-043 takes the same
  stance as ADR-027 step 7 — fill INVALID slots only, no live
  eviction under the page write lock, even when an α-dominated
  live neighbor would be replaced by a planned candidate.
  Reviewer should confirm that this doesn't regress the Vamana
  α-invariant badly enough to require a rebuild sooner than
  the `inserted_since_rebuild`-style drift trigger already
  tracks.
- **Hot/cold ordering (step 6).** When a hot page and its
  corresponding cold rerank page both need updates, the ADR
  says cold lock is acquired after the hot lock for the same
  element and released before moving to the next hot page in
  block order. Confirm this doesn't produce lock-chain
  cycles under concurrent insert (ADR-042 step 5 writes hot
  pages only, but the new-node append in step 2 may touch the
  cold chain too — is there a hidden ordering conflict?).
- **Medoid deferral (step 10).** Setting
  `needs_medoid_refresh` under a narrow metadata exclusive
  lock after all data-page writes commit — reviewer should
  confirm that this does not violate step 7's "no metadata
  exclusive during pass 2" rule. The intent is that the
  metadata flip happens after pass 2 completes, between pass
  2 and pass 3.
- **Read-only replan (step 8).** Same shape as ADR-027 step 6
  and ADR-042 step 7. If all three ADRs share this rule, is
  there a case for a shared helper ADR that captures the
  "replan-before-retry across ordered passes" pattern once,
  rather than repeating the decision in every mutation ADR?
  Flag as a meta-question.

## Questions to answer

- **α-pruning at fill time** — the ADR says fill uses
  `RobustPrune` over (existing live neighbors ∪ planned
  candidates), but the fill-only posture forbids evicting live
  neighbors. So `RobustPrune`'s output, when clipped to
  fill-only slots, may produce a set that violates α.
  Concretely: is "run `RobustPrune` and then only apply the
  subset of its output that lands in INVALID slots" still
  correct, or should fill fall back to nearest-first
  selection (ignoring α) because α cannot be enforced
  without eviction?
- **Vacuum racing live insert** — ADR-042's stale-target
  retry path assumes a planning-pass snapshot may see a
  neighbor that a concurrent vacuum has unlinked. Does ADR-043
  step 5 leave enough information in the tuple (e.g.,
  INVALID slots, generation counters) for the insert retry
  to detect this, or does the insert side need a separate
  vacuum-generation check?
- **Pass 3 invariants** — reviewer should confirm that pass 3
  operates only on data already stable after pass 2, and that
  no concurrent insert or scan can promote an orphan back to
  live state between pass 2 and pass 3.

## Dependencies

- ADR-034 (PROPOSED at review time).
- ADR-027 (ACCEPTED, reference protocol).
- ADR-042 (PROPOSED companion — see packet 11002).

## Companion packets

- `review/11001-diskann-task17-plan/` — task 17 plan.
- `review/11002-adr042-vamana-insert-lock-ordering/` —
  ADR-042 draft (insert counterpart).
- `review/11004-diskann-build-algorithm-design/` — build
  design, which describes medoid approximation and supplies
  the rule that step 10 defers to rebuild.

## Definition of ready (for ADR-043 → ACCEPTED)

- Reviewer confirms the 10-rule decision covers every write
  path that any phase-5 implementation could take, or lists
  the specific write paths not yet covered.
- α at fill time is either explicitly enforced or explicitly
  deferred to rebuild.
- The hot/cold ordering inter-check with ADR-042 is cleared.
- Open questions above are resolved or deferred with a
  follow-up ADR number.
