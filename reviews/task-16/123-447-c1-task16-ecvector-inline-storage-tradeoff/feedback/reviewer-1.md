## Feedback: ecvector inline-storage tradeoff — ACCEPTED, mitigation space under-explored

Verified against `7fa40d3`. The measurement is clean; the write-path
penalty is real; the question "what else could we do?" is the one
worth spending feedback on.

### What's right

- **The tradeoff is measured, not guessed.** Buffer-cache pressure
  (`2.86% → 305%` of `shared_buffers`), WAL per 1k-row steady batch
  (`4.0MB → 14.3MB`, `3.56×`), HOT (`38 → 0`), and build time
  (`-3.87%`, slightly better inline) are the four cells the task-16
  plan asked for. All four landed with honest numbers.
- **Primary mitigation guidance is correct and user-facing.**
  "Keep embedding rows static; move churn metadata to a sibling
  table; join when needed" is the correct structural answer, and
  it's the lever a real workload can adopt without help from the
  extension. Good framing.
- **Packet explicitly refuses to pick the product default.** Leaves
  the policy choice — inline-default vs expert-lever — open for a
  later decision. That's right; the lever has enough surface area
  that it shouldn't be decided inside a measurement packet.

### Mitigation space — what I would want pushed on

The packet names `fillfactor` in one line and moves on. Given how
load-bearing the inline lever is for task-16's serious-lane story,
the mitigation analysis deserves more. The write-path penalty has
several candidate mitigations with materially different cost
profiles. Worth a second measurement packet before task-16 merges.

1. **`SET STORAGE EXTERNAL` instead of `PLAIN`.** The packet-`441`
   decode-bucket collapse was `1386us → 1us` — that was the
   `varlena` detoast **and** pglz decompression cost combined.
   `EXTERNAL` keeps the column TOASTed (preserves the
   small-heap-tuple UPDATE shape and therefore HOT) but skips
   compression. A TOAST fetch still costs a TOAST-table lookup, but
   it skips the decompression step, which is likely the dominant
   piece of the `1386us`. If `EXTERNAL` recovers most of the
   serious-lane win without the `3.56×` WAL cost, that is a
   dramatically better product default than `PLAIN`.
   **Worth measuring as a separate cell: `ecvector` with
   `attstorage='x'` on the same fixture.** This is the single
   highest-value follow-up measurement I can identify from this
   packet.

2. **`fillfactor < 100` on the inline table.** HOT-update probability
   is gated on "does the new tuple version fit on the same page as
   the old one." With 6 KB tuples and default `fillfactor = 100`,
   pages are near-full after insert and HOT can't land. Dropping to
   `fillfactor = 80` or `70` leaves ~1.6-2.4 KB free per page, which
   may be enough to restore HOT for moderate non-indexed-column
   churn without giving up the inline read win. The packet says
   this is "not the primary answer" — likely true, but also
   unmeasured. At ~20% extra heap pages, it's cheap to measure and
   may close most of the HOT gap on realistic churn patterns.

3. **Inline-f32 inside the index, not the heap.** This is the
   architectural alternative and the one that isolates heap-row
   churn from rerank storage entirely. Concept:
     - Heap column: `ecvector` with default (TOASTed) storage.
     - Index tuple: gain a cold-page inline-f32 payload read only
       during rerank (same shape as pre-`442` `persisted_source_column`
       but owned by the index, not a user column).
   Total bytes are ~the same as inline-heap `ecvector`, but they
   live in index pages. Consequences:
     - Row UPDATE churn on non-embedding columns reverts to the
       `468`-heap-page profile — small tuples, HOT works, normal WAL.
     - The inline-f32 bytes still exist somewhere; they're in the
       index, which is rebuilt on CREATE INDEX and vacuumed on a
       separate schedule. Index-build cost goes up because now the
       index writes 4×dim bytes per entry in addition to the quant
       codes, but the index already pays one write per entry at
       build and one read per rerank.
     - The "rerank reads from heap tuple" fast-path becomes "rerank
       reads from the index's cold page" — same cache class (index
       pages are well-behaved in `shared_buffers`), no
       cross-relation fetch.
   This is worth an ADR exploration, not a quick measurement. But
   it's the cleanest answer to "how do you get the serious-lane win
   without penalizing row churn" on architectural grounds.

4. **Vertical partitioning at the application level.** Structural
   equivalent of #3 done by the user: `(id, embedding)` in one
   table, `(id, metadata…)` in another. This is the packet's
   existing guidance. Worth noting explicitly that this *is* a form
   of decoupling — the question is whether the extension should
   ship something that does it automatically or require users to
   do it themselves. Today it's user-side only.

5. **Reject the operating point entirely.** If rerank-from-heap is
   the whole reason we pay the inline cost, and index-side rerank
   (option 3) costs the same bytes but avoids the churn penalty,
   the right answer may be to deprecate inline-`ecvector` as a
   product knob entirely and move the heap-f32 rerank story to an
   index-side payload. Not saying do this, but it's the option at
   the other end of the design spectrum and should be named.

### Concerns on the measurement itself

1. **First-batch WAL numbers dismissed as "noisier."** The packet
   quotes the second batch only. The steadier number is the right
   one to quote, but reporting the first-batch numbers would let a
   reader see how much variance there is batch-to-batch. Without
   them, "3.56× WAL" reads as a fixed multiplier when it may be a
   point estimate in a noisy distribution.
2. **Single shared-buffers setting.** `shared_buffers = 128MB` on a
   50k corpus with inline tuples gives a `305% / shared_buffers`
   heap working set. A deployment-representative config likely has
   `shared_buffers` in GB. The multiplier changes with buffer
   sizing, so the "buffer pressure" framing may be alarmist for
   realistic deployments.
3. **No `fillfactor` or `attstorage='x'` cells.** These are cheap
   to measure (same fixture, two extra surfaces) and they're the
   two most plausible cheap mitigations. Missing them leaves the
   "inline helps / inline hurts" framing binary when it may be
   three-way or four-way.
4. **Update probe touches a single 4-byte `integer` column.** That
   is the right worst-case (the UPDATE is forced to rewrite the
   whole 6 KB row just to change a small int), but it's not
   representative of all update workloads. An UPDATE that also
   touches a larger column will have a smaller relative multiplier
   because the "rewrite the whole row" cost is amortized. Worth a
   one-line note that `3.56×` is the tip, not the average.

### Questions for coder-1

1. **Is the serious-lane win from "in-heap bytes" or from
   "un-compressed / un-detoasted bytes"?** The packet-`441` decode
   bucket collapse is the load-bearing evidence; decomposing it
   into the detoast and decompress components would tell us whether
   `EXTERNAL` (no compression) captures most of it.
2. **On the inline probe, did the UPDATE touch the embedding
   column at all, or only `touch`?** Packet says only `touch`;
   worth confirming since that's the worst-case shape.
3. **Was `fillfactor` left at default 100 on both surfaces?**
   The packet doesn't say. If yes, that's the obvious next cell.

### Call

Accepted as "the measurement is right, the mitigation story is
incomplete." The three-way `PLAIN` / `EXTERNAL` / `EXTENDED`
comparison at minimum, and ideally a `fillfactor` sweep at
`EXTERNAL`, should land before task-16 decides what product
default `ecvector` carries. The index-side rerank payload
alternative is ADR-scale and out of scope here, but worth naming
as the architectural option space.

The packet answers "inline works, inline has a cost." It does not
yet answer "is inline the best way to pay for that win." That
second question is what the next measurement packet should close.
