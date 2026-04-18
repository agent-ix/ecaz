# Review Request: ADR-042 Vamana Live Insert Lock Ordering

Branch: `adr034-diskann-access-method`

Scope:
- `spec/adr/ADR-042-vamana-insert-lock-ordering.md`

## What this slice is

Draft ADR-042 (status: PROPOSED) covering the lock-ordering
protocol for live insert on the upcoming `tqdiskann` access
method. Analog of ADR-026 for HNSW, adapted to single-layer
Vamana and α-pruning.

No code lands under this slice. ADR-042 must move to ACCEPTED
before task 17 phase 4 (insert implementation) begins.

## What changed

New file: `spec/adr/ADR-042-vamana-insert-lock-ordering.md`.
Status PROPOSED.

Decision encodes nine rules:

1. Traverse read-only first (greedy from medoid entry point).
2. Append new node under one data-page `EXCLUSIVE` lock.
3. Release append lock before backlink work.
4. Collect backlink targets, sort by `(block_number,
   offset_number)`.
5. Rewrite one page at a time in ascending block order.
6. α-pruning (`RobustPrune`) runs pure-function inside the page
   write window; inputs all materialized outside.
7. Stale-target retry via read-only replanning between ordered
   passes.
8. Metadata-page `EXCLUSIVE` lock only after data-page writes
   complete.
9. First-insert bootstrap stays under metadata-page lock.

Delta from ADR-026:

- No per-layer ordering (single-layer graph).
- `RobustPrune` under page lock replaces top-M score eviction.
- Entry-point medoid does not migrate at live-insert time.
- Smaller metadata-mutation surface during insert.

## Review focus

- **α-pruning under page lock (step 6).** The claim is that
  `RobustPrune` is bounded by `R` candidate-vs-candidate score
  evaluations and does not acquire further buffer locks or read
  other data pages. Reviewer should confirm this is achievable
  when the target's neighbor list refers to element TIDs on
  *other* pages — i.e., can `RobustPrune` score candidates
  whose codes live elsewhere without paging them in under the
  held exclusive lock?
- **Stale-target retry (step 7).** The protocol mirrors
  ADR-026 step 7, but Vamana's stale detection condition is
  richer (concurrent inserter may have both added the new TID
  and pruned a candidate we expected). Is the minimum retry
  payload `(target_element_tid)` plus the new-node backlink
  sufficient, or does retry need to carry the target's
  neighbor-list snapshot from planning?
- **Metadata narrowness (step 8).** ADR-042 claims the
  metadata write scope during insert is narrower than
  `tqhnsw`'s because there is no entry-point TID mutation. Does
  this match the `needs_medoid_refresh` flag in ADR-043 step 10
  — i.e., vacuum writes it but insert never does?
- **Retry bound.** ADR-042 proposes capping retries per insert
  with a loud warning on exceed. Is the current numeric bound
  from ADR-026 applicable (it doesn't specify one explicitly
  either), or should this ADR pin a concrete number?

## Questions to answer

- **Scoring during read-only planning** — task 17 phase 4 will
  reuse the scan greedy-search helper for candidate discovery.
  Should ADR-042 add an explicit "read-only planning uses the
  same scoring wrapper as scan (PqFastScan + optional binary
  prefilter + heap-f32 rerank)" rule, or is that an
  implementation detail below the ADR's abstraction level?
- **Cold rerank-payload interaction** — ADR-042 does not discuss
  cold page chain writes during insert. Does the new node's
  cold rerank payload append happen before step 2 (and land in
  the same hot-page write?) or separately? ADR-042 leaves this
  implicit; please flag if it should be spelled out.
- **Deadlock proof** — Reviewer should stress-test the rule
  set against: two concurrent inserts choosing overlapping
  backlink-target pages; insert racing vacuum's pass 2 on the
  same target (ADR-043 step 5's fill-only rule is the
  counterpart); insert racing vacuum's pass 3 finalization.

## Dependencies

- ADR-034 (ACCEPTED or still PROPOSED at review time — if
  ADR-034 is still PROPOSED, ADR-042 is a conditional ADR that
  only binds if ADR-034 is accepted).
- ADR-026 (ACCEPTED, reference protocol).

## Companion packets

- `review/11001-diskann-task17-plan/` — task 17 plan.
- `review/11003-adr043-vamana-vacuum-lock-ordering/` — ADR-043
  draft (vacuum counterpart; shares the "one page exclusive at
  a time, replan read-only" invariant with this ADR).
- `review/11004-diskann-build-algorithm-design/` — build design.

## Definition of ready (for ADR-042 → ACCEPTED)

- Reviewer confirms the 9-rule decision covers every write path
  that any phase-4 implementation could take, or lists the
  specific write paths not yet covered.
- Open questions above are either resolved or explicitly
  deferred with a follow-up ADR number.
- Deadlock proof exercise (two concurrent inserts, insert vs
  vacuum) passes reviewer scrutiny.
