# Review Request: ADR-046 Review Prep (Gaps Blocking ACCEPTED)

Branch: `adr034-diskann-access-method`
Author: coder-2
Target: `spec/adr/ADR-046-vamana-insert-lock-ordering.md` (PROPOSED)

## What this packet is

Pure docs packet. Identifies gaps in ADR-046's current PROPOSED
text that need answers before the ADR can move to ACCEPTED and
unblock Phase 7 (insert pgrx wiring) + Phase 8B (pgrx vacuum).

Not a rewrite. Each gap is one question with an expected answer
shape. Reviewer can resolve inline or push back.

## Why ACCEPTED matters

- Phase 7 pgrx `aminsert` needs the lock-ordering protocol frozen
  so the pgrx callback can be written without re-guessing which
  locks it's allowed to hold when.
- Phase 6A's scan shell does not depend on ADR-046 (scans are
  read-only). Phase 6B does not depend on it either. Only Phase 7
  + concurrent Phase 6B behavior depend on ADR-046.
- ADR-047 vacuum (packet 11025) shares the ordered-page-pass
  harness invariant with ADR-046; answers on retry caps and
  concurrency with vacuum are load-bearing for both.

## Gaps

### G1 — Overflow-heaptid chain not addressed on insert

Step 2 says the new node's append "may allocate a fresh page, but
holds only that single data-page write lock." The slim-tuple
layout (ADR-045 Decision 4) includes a `has_overflow_heaptids`
flag and the V1 design mentions a future overflow chain for live
insert (handled by a HOT-update-like secondary TID path).

**Question.** When an insert binds a heap TID to an existing
element tuple (e.g., the same vector is already indexed and the
new row is a HOT/update), does ADR-046 govern the overflow chain
write, or is that a separate ADR? If governed here, which step
covers it — step 2 (append, but it's not an append) or step 5
(rewrite existing tuple in ascending block order)?

**Expected resolution.** One paragraph + one step. Most likely:
"overflow heaptid installation follows the same ordered-page-pass
rule as step 5; if the target tuple's page was not already in the
backlink target set, fold it into that ordered pass."

### G2 — Cold rerank payload chain not addressed on insert

ADR-045's payload-flag design includes `PAYLOAD_FLAG_COLD_RERANK_
PAYLOAD`. Cold rerank payloads live on a separate chain from the
hot node pages. ADR-047 (vacuum) explicitly addresses the cold
chain under step 1 and 6. ADR-046 (insert) does not say where the
cold payload is written.

**Question.** On insert, under what lock is the cold rerank
payload allocated + written? Step 2 only mentions the hot append
page.

**Expected resolution.** A sub-step under 2 ("and allocate the
cold rerank payload under the cold chain's append lock, released
before step 3"), or an explicit "cold rerank payload is written
in the same GenericXLog as the hot append" clause.

### G3 — Stale-target retry cap is not named

Step 7 describes the stale-target retry loop; Consequences §2
notes it is "theoretically unbounded" and offers "cap retries per
insert with a loud warning on exceed" as mitigation. ADR-026 has
a concrete cap (check `src/am/insert.rs` for the constant).

**Question.** Does ADR-046 adopt ADR-026's retry cap verbatim, or
does Vamana insert have a different cap (e.g., because α-prune
under the page lock is slower)?

**Expected resolution.** Name the cap. Most likely the same as
ADR-026, with a note "reusing ADR-026's cap; revisit if recall or
p99 latency warrants a Vamana-specific value."

### G4 — Entry-point drift-trigger mechanics

Step 8 mentions "optional drift-trigger fields" on the metadata
page; the Phase 5C-2 metadata page schema includes
`needs_medoid_refresh: bool` and `inserted_since_rebuild: u64`.
Decision 9 ("No entry-point TID mutation during live insert")
commits to not migrating the medoid, but it doesn't say how
drift is detected or what action the trigger produces.

**Question.** What condition flips `needs_medoid_refresh` during
live insert? Options:
- Never during insert (only at vacuum per ADR-047 step 10).
- When `inserted_since_rebuild` crosses a threshold.
- Never during insert, with a post-insert "did we drift far from
  the medoid" check delegated to a scheduled maintenance pass.

**Expected resolution.** One sentence in step 8: "insert flips
`needs_medoid_refresh = true` iff X" or "insert does not flip
`needs_medoid_refresh`; that flag is owned by vacuum (ADR-047
§10) and by the rebuild scheduler."

### G5 — Concurrency with vacuum not explicit

The ADR implies vacuum and insert can run concurrently (stale-
target retry is the escape valve when they collide). ADR-047
likewise implies concurrent insert via its own retry loop.

**Question.** Is there a documented invariant "insert and vacuum
may run concurrently, and each tolerates the other via stale-
target retry"? Currently it's implied but not stated. A single
sentence in Context or Decision would freeze the design intent
and prevent a future reader from concluding one must serialize
against the other.

**Expected resolution.** One sentence. Most likely in Context:
"concurrent insert + vacuum is supported via the shared stale-
target retry shape (ADR-046 step 7 + ADR-047 step 8)."

### G6 — α-prune cost bound references the slim-tuple layout

Step 6 ("α-pruning inside the page write window is bounded")
says the prune inputs are "either already in the target's on-
page tuple or came in from the read-only planning pass." This is
correct for the slim-tuple layout (ADR-045 Decision 3) because
PqFastScan codes are on the tuple.

**Question.** Should the ADR add a forward reference to ADR-045
Decision 3 here to lock the invariant "tuple is self-sufficient
for prune scoring"? Minor but makes the page-locality argument
self-verifying.

**Expected resolution.** One inline reference added to step 6:
"…per-node PqFastScan codes cached on the page (see ADR-045
Decision 3, slim tuple)…"

## Non-gaps (affirming choices)

- Fill-only vs. RobustPrune split on insert (step 6) is coherent:
  RobustPrune runs over the target's existing neighbor list + the
  new candidate, all readable from the on-page tuple. No eviction
  of live neighbors without the planning trail.
- Metadata-last invariant (step 8) matches ADR-026 cleanly and
  mirrors ADR-047 step 10.
- First-insert bootstrap (step 9) reuses the `tqhnsw` path,
  correct.
- Single-layer topology justifies dropping ADR-026's layered
  dance; this is spelled out in Context.

## Suggested action

1. Reviewer resolves G1–G5 inline in the ADR (short paragraphs or
   sub-steps). G6 is a one-line forward reference.
2. Flip status to ACCEPTED once G1–G5 have answers.
3. Cross-link ADR-045 Decision 3 from ADR-046 step 6 (G6).

## Dependencies

- **ADR-045 ACCEPTED** — provides the slim-tuple invariant G6
  references.
- **ADR-026 + ADR-027** — ADR-046 borrows the ordered-page-pass
  invariant and the retry loop shape from these. G3's resolution
  likely defers to ADR-026's cap.

## Companion packets

- **11002** — original ADR-046 draft packet.
- **11025** — ADR-047 review prep (parallel gap list for vacuum).
- **11021** — Phase 8A vacuum primitives (complementary tests
  that assume ADR-047's three-pass structure).

## Definition of ready (for the ADR, not this packet)

- G1–G5 have explicit answers in the ADR text.
- G6 resolved or explicitly waived.
- No pending cross-reference to ADR-045 left dangling.
- Reviewer signs off; status changes to ACCEPTED.

## Not doing in this packet

- Rewriting the ADR. This is a gap list, not a proposal.
- Rules for multi-tenant concurrent-insert-in-bulk scenarios (bulk
  build does not run live; that's already stated in step 9).
- Live medoid migration (explicitly out-of-scope per Consequences
  §3).
- Implementation sketches for Phase 7 — those belong in a Phase 7
  design packet once ADR-046 is ACCEPTED.
