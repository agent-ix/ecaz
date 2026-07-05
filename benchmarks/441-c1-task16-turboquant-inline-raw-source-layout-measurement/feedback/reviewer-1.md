## Feedback: Inline raw-source layout measurement — ACCEPTED, and this is the real task-16 signal

Verified against:

- packet `440`'s supported `source_raw` baseline (q200 `4.838ms`)
- two new corpora:
  - `tqhnsw_real_50k_tq_rawonly_corpus` (default TOAST, no sibling
    `source`)
  - `tqhnsw_real_50k_tq_mixed_inline_corpus` with explicit
    `ALTER COLUMN source SET STORAGE EXTERNAL` and
    `ALTER COLUMN source_raw SET STORAGE PLAIN`
- two q200 summaries + two rerank micro-profiles in `tmp/`
- recall summary TSV at `20260419T021226Z_summary_tqhnsw_real_50k_tq_mixed_inline_m16_idx_...`

### What's right

- **Two-control design separates "is raw better" from "is inline
  better".** Raw-only with default TOAST regressed to `5.104ms`
  (`+5.5%` vs `440` baseline). Mixed-inline with `STORAGE PLAIN`
  landed at `3.137ms` (`-35.16%`). Without both surfaces a reader
  would have no way to distinguish "raw-f32 column helps" from
  "inline storage helps" — and the right answer turns out to be
  only the second one. That is a genuinely load-bearing
  measurement decision.
- **Rerank micro-profile tells the whole story.**
  `heap decode` dropped from `1386us` to `1us`. Fetch and dot
  barely moved. That is unambiguous evidence that the
  serious-lane cost was detoast + decode, not the scorer. The
  top-line `-1.701ms` end-to-end matches the `-1352us` rerank-
  total delta — the two numbers are consistent at the ms level,
  which rules out "the savings went somewhere we didn't measure".
- **Recall pinned.** `graph_recall_at_10 = 0.9629` and
  `mean_abs_score_error = 0` — bit-identical to packets `429` /
  `430` / `440`. The layout change does not touch correctness.
  A `-35%` win with zero recall drift is the right shape for
  task 16.
- **Scope is right.** No Rust scan kernel change, no scorer
  change, no default flip. Only heap storage shape + the
  persisted reloption that packet `439` already productized.
  This is exactly the experimental shape the method calls for
  before we commit to a first-class type / storage path.
- **Reverses the offline story cleanly.** Packet `437` showed
  scorer levers barely moved the `heap_f32` lane and named
  heap-rerank as the limiter. Packet `441` proves that diagnosis
  with a surgical experiment — different controls, same
  verdict. That is the measurement chain closing.

### Concerns

1. **n=1 per cell at the biggest claimed delta in task 16.**
   `-35.16%` is large enough to survive the `4.65 – 6.36ms`
   packet-`432` noise envelope by any reasonable margin, so the
   direction is not in question. But a single q200 cell per
   surface should still be reinforced with a second pass before
   this becomes the task-16 headline. The rerank-decode
   `1386us → 1us` is strong corroboration; still, one more q200
   rerun on both surfaces would make the "confirmed" framing
   bulletproof. Cheap insurance.
2. **Storage cost is real and not named quantitatively.** The
   mixed-inline heap grew from `43MB` to `390MB` — roughly
   **9× heap footprint** for `-35%` latency. Total footprint
   (heap+TOAST) is about the same, but *heap page residency*
   is now dominated by `source_raw`. On a large index this
   changes buffer-cache pressure, vacuum cost per page, WAL
   volume on updates to the heap, and index build time.
   Worth a one-liner future-work bullet naming the storage
   cost explicitly so the next packet (or an ADR) can measure
   this as a tradeoff, not just a win.
3. **`STORAGE PLAIN` on `source_raw` only works because the
   row fits.** Avg row is `13135 bytes` with `STORAGE PLAIN
   source_raw` + `STORAGE EXTERNAL source` — close to the
   single-page `8KB` limit with room because `source` is
   externalized. If a future corpus pushes row width up (e.g.
   2048-dim embeddings, or an added metadata column), this
   layout could silently start getting rows TOAST-detoasted
   again. Worth documenting the assumption and maybe an
   assertion / warning when the corpus exceeds a threshold.
4. **`heap fetch` went *up* (`+36us`) on the mixed-inline
   surface.** Small, but real. That's consistent with a larger
   heap page footprint — more cache pressure during random
   page fetches. The `-1385us` decode win dominates, but this
   `+36us` is a signal that the tradeoff is not "pure win" and
   could bite on a smaller latency budget later.
5. **The next step named in §2 is "first-class inline raw-f32
   source path" or "documented/automated storage-policy
   support".** Those are two very different product moves — one
   is code (ADR-043 territory), the other is docs. Worth
   separating them into two explicit future options so the
   discussion about "which way" doesn't treat them as
   interchangeable. A native `tqvec` column that always lives
   inline has different deployment implications than "tell
   users to run `ALTER COLUMN source_raw SET STORAGE PLAIN`
   in their migration scripts".

### Questions for coder-1

1. **Does q200 ×3 reproduce `~3.1ms` on the mixed-inline
   surface?** With a delta this large, even a single rerun is
   low-risk — but "the cleanest serious-lane readout in task
   16 so far" should be rebuilt from at least 2 runs before it
   becomes the merge justification.
2. **Is the `+36us` heap-fetch regression consistent across
   reruns?** If so, what's the expected asymptotic as corpus
   size grows? On 50k it's noise; on 5M it might dominate.
3. **What is the index build time delta between the raw-only
   control and the mixed-inline candidate?** Build-time
   regressions on heavy-write workloads can matter as much as
   scan-time wins, and a `9×` heap-footprint change usually
   shows up in build cost. Not captured in this packet; worth
   a one-liner before any default flip.
4. **Does the ADR-043 `tqvec` native-type path aim to
   guarantee inline storage** (i.e. reimplement what `STORAGE
   PLAIN` forces), or is it orthogonal? This packet's result
   is arguably *the strongest argument for ADR-043* because
   the measurement-based path to this outcome through reloption
   only is "tell users to ALTER STORAGE" — fragile advice.
   Native type solves that. Worth explicitly connecting this
   measurement to the ADR motivation.
5. **Any concern about `STORAGE PLAIN` interacting badly with
   parallel bulk INSERT / COPY on wide rows?** Postgres
   generally handles it, but the heap-page contention profile
   changes. For a serving index that might not matter; for a
   corpus-building pipeline it could.
6. **Head-to-head vs PqFastScan on the same lane?** Task 16's
   stated outcome goal is "narrow the TurboQuant vs PqFastScan
   latency gap on the 50k warm real seam". Packet `441` brought
   the serious lane to `3.137ms`, but no cell in the 422–441
   arc has put this surface next to PqFastScan on the same
   corpus, recall target, and runtime. Without that cell, task
   16 closes as "big internal improvement, gap question
   answered by inference" rather than by direct measurement.
   Strongly recommend one q200 cell on PqFastScan against
   `tqhnsw_real_50k_tq_mixed_inline_corpus` (or an equivalent
   PqFastScan surface) before task 16 merges — the framing
   question deserves a measurement, not an assumption.

### Call

Accepted as the most important measurement in the task-16 arc.
The serious-lane bottleneck was heap-source layout, not scorer
math — packet `437` predicted that, and this packet proves it
with a `-35%` runtime delta and a rerank-decode micro-profile
that collapses the cost bucket entirely. The next implementation
target should be first-class inline raw-source storage, not
another scorer experiment — that matches both this measurement
and the direction ADR-043 was already pointing.

Two must-haves before this lands as a default:

- **n≥2 q200 rerun** on the mixed-inline surface — the delta is
  large enough that one confirming run is plenty, but one
  confirming run is not zero.
- **Storage cost named as an explicit tradeoff** — `9×` heap
  footprint is a real cost; the measurement should not carry
  forward without naming it.

The method continues to pay off. The offline scorer story
(packet `433`) undershot lever 4 and missed the real limiter
entirely; the live matrix (packet `437`) pointed at heap rerank;
and this packet closed the loop. Proof, not assumptions, keeps
being the right call.
