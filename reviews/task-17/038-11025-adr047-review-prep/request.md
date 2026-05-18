# Review Request: ADR-047 Review Prep (Gaps Blocking ACCEPTED)

Branch: `adr034-diskann-access-method`
Author: coder-2
Target: `spec/adr/ADR-047-vamana-vacuum-lock-ordering.md` (PROPOSED)

## What this packet is

Pure docs packet. Identifies gaps in ADR-047's current PROPOSED
text that need answers before the ADR can move to ACCEPTED and
unblock Phase 8B (pgrx vacuum callback).

Phase 8A (11021) already lands the tuple-level vacuum primitives
(`mark_deleted`, `strip_dead_primary_heaptid`, `repair_neighbors`)
that ADR-047's three passes orchestrate. The primitives do not
depend on the ADR being ACCEPTED because they operate at a layer
below lock ordering. Phase 8B's pgrx callback does depend on it.

## Gaps

### G1 — Pass 1 does writes; the three-pass / two-pass split is ambiguous

Step 1 ("Pass 1 — dead-heap-TID discovery") ends with "Strip the
dead heap TIDs under a narrow per-page BUFFER_LOCK_EXCLUSIVE."
That is a write. Pass 3 (step 9) is also a write ("Mark
`deleted = true`"). So vacuum has two write passes (1 and 3) plus
a write pass 2 — three total, which matches the ADR's own
numbering. Fine.

But the text of step 1 reads like a pure discovery pass ("dead-
heap-TID discovery") and only mentions the EXCLUSIVE lock as an
aside. Phase 8A's `strip_dead_primary_heaptid` expects a distinct
write under a page lock, not a discovery read.

**Question.** Is pass 1 one pass (share-scan + per-page exclusive
strip interleaved) or two (all reads first, then all strips)?
The primitive names suggest the former; the step text leaves it
ambiguous.

**Expected resolution.** One sentence clarifying. Most likely:
"pass 1 interleaves per-page discovery (SHARE) and per-page
strip (EXCLUSIVE) in a single ordered block scan; the dead-TID
set is observed and mutated within one GenericXLog per page."

### G2 — Candidate generation when the medoid is in the delete-set

Step 3 says replacement-candidate generation uses greedy search
"starting from the live medoid entry point." Step 10 says if the
medoid TID is in the delete-set, pass 2 *does not migrate it*.

**Question.** Does step 3 use the dead medoid TID as the entry
point (producing a search that will return quickly because the
medoid tuple still exists, just has INVALID primary_heaptid), or
does it pick a live fallback? If the latter, which one — first
live TID in block order? A configurable fallback list?

**Expected resolution.** One sentence. Most likely: "if the
entry-point TID's own element is in the delete-set, read-only
planning uses the lowest-block live element as a temporary entry
point. The medoid refresh is still deferred per step 10."

### G3 — Pass 3 orphan detection cost + algorithm

Step 9 says pass 3 identifies "elements that are now orphans
(all heap TIDs dead, no live inbound neighbor references)."

**Question.** How is "no live inbound neighbor references"
computed efficiently? The obvious implementation is O(N·R): scan
every live element's neighbor list, build an in-ref count per
element. The ADR does not specify. For R=32 and N=10M that is 3×
10^8 edge checks — bounded but not trivial.

**Expected resolution.** One paragraph. Options:
- (a) "Pass 3 maintains an in-ref count per element during pass 2,
  updating as neighbor lists are rewritten; orphan = in-ref
  count 0 + heap TID dead."
- (b) "Pass 3 is a second ordered block scan (SHARE) that reads
  every live element's neighbor list and accumulates an in-ref
  bitmap keyed on TID. Orphan = not in bitmap + heap TID dead.
  Cost O(N·R) edge checks, linear in graph size."
- (c) "Pass 3 defers orphan detection; tombstones are set based
  on heap-TID-only criteria and the graph may retain edges to
  tombstoned elements. Scan filters at read time."

Pick one. My read is (b) is simplest and matches the fill-only
posture. (a) couples pass 2 + pass 3 tightly.

### G4 — Cold rerank payload lifecycle at pass 3

Pass 3 flips `deleted = true` on the hot tuple. The cold rerank
payload chain is addressed in passes 1 and 2 (steps 1, 6) but
not in pass 3.

**Question.** When pass 3 tombstones a hot element, is the
corresponding cold rerank payload chain entry also freed? If so,
under which lock? If not, under what condition is it freed (next
vacuum run? never until rebuild?).

**Expected resolution.** One paragraph. Most likely: "cold
rerank payload for a tombstoned element is freed in the same
GenericXLog as the pass-3 hot-tuple flip, via cold-chain
exclusive lock acquired after the hot lock in step 9."

### G5 — Retry cap on read-only replan

Step 8 describes the retry-on-stale loop; Consequences §3 notes
it is "theoretically unbounded" but "bounded in practice." ADR-
027 likely has a concrete cap.

**Question.** What is the retry cap? Same as ADR-027, or
different for Vamana vacuum (e.g., because α-aware repair
planning is more expensive)?

**Expected resolution.** Name the cap. Same gap shape as ADR-
046 packet (11024) G3.

### G6 — Concurrency with insert not explicit

Same gap as ADR-046 G5, but from the vacuum side. The ADR
implies concurrent insert (retry loop is the escape valve) but
does not state "vacuum and insert may run concurrently" as an
invariant.

**Expected resolution.** One sentence. Most likely in Context:
"concurrent insert + vacuum is supported; ADR-046's insert side
uses the same ordered-page-pass harness and stale-target retry
shape."

### G7 — `needs_medoid_refresh` flag ownership

Step 10 writes `needs_medoid_refresh` on the metadata page.
ADR-046 also references drift-trigger fields on the metadata
page. Two writers need a documented owner.

**Question.** Can both insert (ADR-046) and vacuum (ADR-047)
flip `needs_medoid_refresh`? If so, is the flag monotonic
(once set, stays set until rebuild clears it)? What clears it?

**Expected resolution.** Cross-reference paragraph. Most likely:
"ADR-047 pass 3 sets `needs_medoid_refresh = true` when the
entry-point element is tombstoned. ADR-046 step 8 does not flip
the flag (see ADR-046 G4 resolution). The rebuild callback
(future ADR) clears it. Flag is monotonic true→false only on
rebuild."

### G8 — RobustPrune in planning vs. fill-only in writes

Step 3 says planning runs `RobustPrune`; Consequences §1 says
"α-aware candidate selection preserves Vamana's diversity
invariant." Step 5 says "Live neighbors are never evicted under
the page write lock" — fill-only.

**Question.** These are not contradictory, but the ADR should
spell it out: "RobustPrune produces the candidate list during
read-only planning; the write phase picks candidates from that
list *only for INVALID or dead slots*. Live slots are untouched."
Currently a reader has to infer the composition from Decision
§3 + §5.

**Expected resolution.** One sentence clarifying the composition
at step 5.

## Non-gaps (affirming choices)

- Fill-only posture is the right conservative default; ADR-045
  Decision 3 (fixed-length tuples) guarantees `update_raw_tuple`
  soundness, verified by Phase 8A's VC-009.
- One data-page EXCLUSIVE at a time + ascending-block order
  matches ADR-027 cleanly.
- Metadata lock taken last, after all data-page writes, prevents
  the known mixed-order deadlock.
- Medoid migration deferred to rebuild is consistent with
  ADR-046's "no entry-point mutation during insert" stance.

## Suggested action

1. Reviewer resolves G1–G8 inline (G1, G2, G5, G6, G8 are one
   sentence; G3, G4, G7 warrant a paragraph).
2. Flip status to ACCEPTED once G1–G7 have answers (G8 is
   clarification, not a new decision).
3. Cross-link ADR-045 Decision 3 (fixed-length tuples) from
   step 5 for the same reason as ADR-046 packet G6.

## Dependencies

- **ADR-045 ACCEPTED** — fixed-length invariant for
  `update_raw_tuple` soundness.
- **ADR-027** — ordered-page-pass + retry shape.
- **ADR-046** — parallel insert-side ADR; G6 and G7 resolutions
  need to be consistent across both.

## Companion packets

- **11003** — original ADR-047 draft packet.
- **11024** — ADR-046 review prep (parallel gap list for insert).
- **11021** — Phase 8A vacuum primitives (implements the
  per-tuple half of passes 1/2/3).

## Definition of ready (for the ADR, not this packet)

- G1–G7 resolved in the ADR text.
- G8 either resolved or explicitly waived.
- Status flipped to ACCEPTED.
- ADR-046 G5 (concurrency) resolved consistently.

## Not doing in this packet

- Rewriting the ADR. This is a gap list.
- Phase 8B pgrx callback sketch — belongs in a design packet
  once ADR-047 is ACCEPTED.
- Rules for live medoid migration — explicitly out of scope per
  Decision §10.
