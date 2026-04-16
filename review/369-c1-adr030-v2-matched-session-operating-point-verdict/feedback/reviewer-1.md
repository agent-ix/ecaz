## Feedback: ADR-030 v2 Matched-Session Operating-Point Verdict

Read the cited recall tables in
`review/362-c1-adr030-v2-real-50k-m16-runtime-validation/request.md:86-87`
and
`review/363-c1-adr030-v2-pgvector-size-and-runtime-baseline/request.md:123-139`,
the SQL-mean tables from packet 368 at lines 133-139, and the
operating-point cross-tables here at lines 118-154.

### What's right

- **Crossing SQL means against direct recall is the right
  next step.** 368 alone leaves the branch with two separate
  tables and an easy-to-overclaim "tqvector faster, pgvector
  more accurate" narrative. Asking the explicit
  same-latency-budget and same-recall-target questions is the
  actual product-facing decision surface.
- **Interpretation is honestly asymmetric.** Lines 162-176
  name exactly what the measured data supports: tqvector owns
  a narrow sub-`1.6ms` pocket, pgvector dominates above, and
  tqvector does not reach pgvector's measured recall floor.
  That shape — two disjoint regions, not a smooth Pareto
  slope — is correctly called out instead of being softened.
- **Scope is correctly restricted to the measured points.**
  Both cross-tables (lines 118-123, 126-132) explicitly use
  "best measured point within budget" rather than
  interpolating, so the packet does not invent data between
  `ef_search` values.
- **Follow-up options at lines 199-205 are framed as a product
  choice rather than a default engineering direction.** "Push
  recall higher without losing the sub-`1.6ms` corner" vs.
  "frame grouped-v2 as latency-first" is the right fork; the
  packet does not prejudge it.

### Concerns

1. **The tqvector recall table disagrees with packet 368's
   tqvector recall table for the same lane.** Lines 73-81 cite
   packet 363 and use
   `0.9200 / 0.9380 / 0.9400 / 0.9460`. Packet 368 lines 149-154
   cite packet 362 and use
   `0.900 / 0.930 / 0.936 / 0.938` for the same isolated
   grouped `m=16` lane on the same 50-query subset. Both
   citations are internally accurate against their source
   packets, but the source packets themselves disagree by up
   to 2.2 points of Recall@10 (~11 matches out of 500), which
   is an order of magnitude above the 1/500 = 0.002 resolution
   of this sample size. This is load-bearing for the verdict
   in this packet, because several budget / target rows sit
   right on the boundary:
   - At tqvector `ef=40`, the 369 table says `0.9200`; the 368
     table says `0.900`. The claim "tqvector's ultra-low-latency
     corner is roughly `Recall@10 = 0.92 .. 0.938`" (line 167)
     uses the more favorable numbers.
   - At tqvector `ef=128`, 369 shows `0.9400`; 368 shows
     `0.936`. The claim "at ef=128 pgvector already strictly
     dominates" is robust either way, but the margin differs.
   The packet needs to reconcile which tqvector recall table
   is authoritative for this lane and record why the two runs
   diverged. Packet 361 was the "deterministic grouped graph
   build" slice; the recall drift across 362→363 on the same
   corpus/query/M is direct evidence the determinism claim has
   not fully landed, or the two packets ran against different
   corpora/codebooks. Until that is resolved, the operating-
   point verdict is built on a recall surface of unknown
   identity.

2. **"Not reached in the measured tqvector sweep" (lines
   152-154) is weak evidence for a hard verdict.** The
   tqvector sweep stops at `ef=320`. The direct-runtime table
   in packet 363 shows tqvector recall at `ef=320` is already
   only `0.9460`, and the step from `ef=128 → 320` gained only
   0.0060. That curve may well be flattening before the
   pgvector `0.9860` floor, but the packet does not show it.
   Extending the sweep to e.g. `ef=512, 1024` would either
   confirm the "tqvector cannot reach pgvector's recall floor
   on this lane" claim (which is the packet's conclusion) or
   reveal that tqvector reaches `~0.97` at higher `ef_search`,
   which would change the operating-point verdict qualitatively.
   The current verdict relies on absence of data beyond
   `ef=320`, which is a weaker argument than a continued
   sweep.

3. **No variance on either SQL means or recall.** The budget
   / target cross-tables treat the means as point estimates.
   At cells where tqvector and pgvector SQL means are close
   (e.g. tqvector `ef=64 = 1.525ms` vs pgvector `ef=40 =
   1.641ms` vs pgvector `ef=64 = 1.775ms`), the 50-sample
   jitter band is on the order of the difference. "tqvector
   owns below `1.6ms`" is a reasonable summary if the SQL
   means are within ~0.1ms of the true value, but nothing in
   the packet shows that. For a decision-grade operating-point
   claim the cross-tables should either include CIs or use
   median / p50 rather than mean.

4. **The "same-recall target" read at lines 141-154 uses a
   step function over four measured points.** That is correct
   for what was measured, but it treats the nearest-greater-
   measured-cell as the effective cost, which systematically
   overstates both sides' latency cost to hit a target. Small
   issue, but the resulting `4.360ms` for tqvector to reach
   `0.9460` and `6.443ms` for pgvector to reach `0.9980` are
   upper bounds rather than empirical costs. One sentence
   noting this would prevent the table from being read as
   "this is the fastest the system can reach recall X."

5. **Verdict is restricted to `m=16` but not framed that
   way.** Lines 187-196 speak about "the current grouped
   tqvector `m=16` lane" — correct — but the summary bullets
   and the follow-up product-choice list at lines 199-205 drop
   the `m=16` qualifier. ADR-030 v2 supports multiple `m`
   values; the verdict may differ at `m=32` or higher. Either
   scope the follow-up choice explicitly to `m=16` or name
   the `m` sweep as the next open question alongside the
   recall-lift / latency-first fork.

6. **No code, but conclusions feed back into spec / ADR
   decisions.** This packet's operating-point verdict is the
   kind of finding that ADR-030 v2 and any future "latency
   mode vs general replacement" framing will cite. On the
   current state of concerns #1 and #2 (recall identity
   unresolved, sweep not extended), the verdict should not be
   written into the ADR yet; it should remain in the review
   record as a provisional read pending reconciliation.

### Measurement gaps still open

- Reconciled tqvector recall table for grouped `m=16` on this
  subset (concern #1). Blocks the verdict.
- Extended tqvector `ef_search` sweep past `320` to
  empirically establish (not assume) the recall ceiling
  (concern #2).
- Variance / CI on SQL means underlying the cross-tables
  (concern #3).
- One `m != 16` data point to establish whether the verdict
  generalizes (concern #5).

### Final framing

The framing — two disjoint regions rather than a smooth
Pareto slope — is the right qualitative takeaway, and the
honesty of separating "tqvector wins latency below ~1.6 ms"
from "pgvector wins everything above" is worth preserving.
But the quantitative operating-point table is currently built
on (a) recall numbers whose identity conflicts with the
immediately preceding packet for the same lane, (b) a sweep
that assumes rather than demonstrates the recall ceiling, and
(c) means-only latency cells at sample sizes where noise is
comparable to the cited margins. The packet should be read as
a provisional read supporting the "is this lane latency-first
or a general replacement?" product question, not as the
decision that answers it. Until concerns #1 and #2 land, any
ADR language derived from this verdict is premature.
