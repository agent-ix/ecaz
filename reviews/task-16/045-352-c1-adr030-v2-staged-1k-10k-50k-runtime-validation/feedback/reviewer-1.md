## Feedback: ADR-030 v2 Staged 1k/10k/50k Runtime Validation

Read the three sweep tables, the 50k planner probe, and the keep/pivot
call in the request. Also cross-checked against packet 351's live
cutover claim (window=4 hardcoded).

### What's right about the approach

- **Staged before widening.** 1k smoke → 10k isolated read → 50k
  comparison, in that order, is the right operational discipline.
  The explicit user question "was this validated at smaller scales
  before treating 50k as authoritative" was the right pushback and
  this packet answers it directly.
- **Honest keep/pivot.** The pivot call ("grouped-v2 runtime path is
  real and promising on 10k; same shape does not yet carry to a
  convincing 50k operating point") is what the numbers actually say.
  Packet 351 could have been framed as a launching pad for 50k
  benching; instead this packet names that as premature. That's the
  right call.
- **Isolated grouped prefixes per scale.** Building
  `tqhnsw_real_1k_grouped_*`, `tqhnsw_real_10k_grouped_*`, and
  `tqhnsw_real_50k_grouped_m8_idx` as dedicated grouped indexes
  avoids the trap of planner-choice ambiguity between grouped and
  scalar sibling indexes. The note at line 66-67 ("so the planner
  would not choose the scalar sibling index by accident") is the
  right paranoia.
- **Planner-facing SQL probe included.** The direct harness measures
  the scan inner loop; the planner-facing `bench_sql_latency_scratch.sh`
  measures what a real query actually pays. Both are needed. For 50k
  you deliberately narrowed to one ef to avoid burning another long
  batch before the grouped-vs-scalar verdict was clear — that's
  correct economy.
- **Operational note about the scratch extension install is
  appropriately cautious.** Not dropping and recreating the extension
  because it would destroy the loaded real-corpus fixtures, and
  falling back to the existing `tests` schema recall surfaces — that
  is the right trade, stated explicitly.

### What the data actually says

Reading the three sweeps together:

**1k subset (derived from 10k corpus):**

- Grouped tops out at 0.9155@ef=200, exact-quantized ceiling 0.9555.
- Interpretation in request is correct: "useful smoke validation
  only, does not reach the exact-quantized ceiling." Not an
  operating point.
- Side observation: the 0.04-point gap between the grouped sweep and
  the exact-quantized ceiling on a 1k subset is itself evidence that
  the graph + rerank window is leaving ~4% of the attainable recall
  on the table even at ef=200. That's the HNSW exploration budget,
  not a quantization limit.

**10k isolated:**

- Grouped 0.9245→0.9360 vs scalar 0.9310→0.9400 — a 0.4-0.7 point
  trail at matched ef.
- Direct-harness latency: grouped 0.87-2.68ms vs scalar 2.68-8.16ms
  (~2-3x better across the sweep).
- Planner-facing SQL: grouped 4.46-6.74ms vs scalar 5.32-11.55ms —
  the gap shrinks but grouped is still consistently better.
- Verdict is correct: "directionally good."

**50k isolated (the headline):**

- Grouped 0.8560→0.8760 vs scalar 0.8600→0.8940 — grouped trails by
  0.4 points at ef=40 and 1.8 points at ef=200. The gap *widens*
  with ef, not narrows.
- Direct-harness latency: grouped 1.60-4.62ms vs scalar 4.06-4.49ms
  — grouped is only clearly better at the lowest ef (40), and even
  at ef=40 the scalar measurement looks like a cold-cache artifact
  (see concern #1 below).
- Planner-facing SQL at ef=128: grouped 8.996ms / scalar 8.457ms —
  grouped is slightly *worse*.
- Verdict is correct: "not yet a grouped-v2 win."

### Concerns

1. **Scalar 50k latency sweep is non-monotonic — cold-cache
   artifact.** Table at lines 177-184:
   | ef | scalar mean ms |
   | 40 | 4.0557 |
   | 64 | 2.2890 |
   | 100 | 2.9891 |

   ef=40 is *slower* than ef=64. That is not physically plausible for
   a warm scan loop — ef=40 searches less than ef=64. The most likely
   explanation is cold-cache: the ef=40 sweep ran first and paid for
   shared buffer fills, then ef=64 onward ran warm. If the grouped
   sweep also ran cold-first, then the grouped latency column is also
   cold at ef=40 (grouped shows 1.60ms at ef=40 which rises
   monotonically after — actually consistent with cold-first because
   grouped amortizes the cold fill cheaper).

   Practical consequences:
   - Don't read "grouped is clearly better at ef=40" from this table.
     It might be; it might be that grouped paid less cold-cache cost
     than scalar because grouped works on narrower per-page payloads.
     Can't tell from a single pass.
   - The 50k planner probe at ef=128 is a narrow-query cold-state
     read (`--cache-state cold`, `--query-limit 10`). Ten queries is
     too few for a reliable cold read even with variance bands.
     The 8.996 / 8.457 numbers could flip on rerun.

   Suggested follow-up: at least one warm-repeat pass on the 50k
   numbers before anyone draws "grouped is at parity / slightly
   worse" as the durable conclusion. Even one warmup pass per ef
   before the measured pass would remove most of the cold-cache noise
   — the existing `bench_sql_latency_scratch.sh` may already support
   `--cache-state warm`; worth checking.

2. **10k → 50k recall degradation is the signal, not the absolute
   number.** Both grouped and scalar drop from ~0.93 at 10k to ~0.87
   at 50k. That's expected: larger corpus → harder recall at fixed
   ef. But the *gap* between grouped and scalar widens:
   - 10k @ ef=200: scalar 0.9400 - grouped 0.9360 = 0.004
   - 50k @ ef=200: scalar 0.8940 - grouped 0.8760 = 0.018

   The gap 4.5x'd. That means something about the grouped path
   scales worse than scalar. Candidates:
   - window=4 is too narrow at 50k — the true top-10 shifts further
     down in approximate order when the search space is larger, and
     4-slot windowed rerank can't catch candidates that arrive late.
   - the grouped *approximate* score itself (not just the rerank
     step) loses discriminative power at 50k — 4-bit quantization
     distortion compounds over more candidates, so the approximate
     order that feeds the window is noisier.
   - the intended `binary -> grouped -> rerank` pipeline is not
     actually wired as a pipeline yet (coder named this in "Next
     Step" #2): "wire more of the intended `binary -> grouped ->
     rerank` pipeline explicitly instead of treating grouped scoring
     as a mostly standalone scan replacement."

   Test plan suggestion: rerun 50k with window=8 and window=16 and
   see whether the recall gap closes. If it closes → window was too
   narrow (easy fix, probably requires making the window a GUC as
   flagged in 351 feedback). If it doesn't → the approximate order
   is the problem, which is a bigger structural investigation.

3. **Direct-harness vs planner-facing latency divergence.** 10k
   grouped direct-harness is 2-3x better than scalar direct-harness,
   but planner-facing SQL is only ~1.5x better. At 50k, direct-harness
   is close and planner-facing flips slightly against grouped. The
   divergence is:
   - direct-harness measures only the scan inner loop
   - planner-facing adds the fixed overhead: query parse/plan,
     executor setup, tuple projection, ...

   That overhead is *corpus-size-independent*. So as corpus grows,
   the fraction of planner-facing latency spent on the scan loop
   grows. If grouped saves N% of scan-loop time, then on a
   scan-dominated corpus (large, cold), grouped should dominate
   planner-facing too — but it doesn't at 50k. That suggests grouped
   has a fixed per-query *setup* cost that scalar does not
   (codebook/LUT preparation, group-wise transform, ...). Worth
   measuring before widening the window.

4. **`query_limit=50` on 10k, `query_limit=10` on 50k.** The 10k
   numbers are averages of 50 queries; 50k is 10. Variance on 10
   queries is ~√5x higher. For the direct-harness sweeps this is OK
   because `ef_sweep` runs the full 200-query table — but the
   planner-facing 50k probe at ef=128 `query_limit=10` is really noisy
   as a 1.5% delta (8.996 / 8.457) claim. The conclusion "grouped is
   slightly worse on planner-facing latency at 50k" is probably
   right, but the number is not tight.

5. **50k exact-quantized ceiling is 0.8560 — matches grouped ef=40.**
   The grouped sweep recall at ef=40 is 0.8560, exactly the
   exact-quantized ceiling. That can mean one of two things:
   (a) grouped is already at the quantization ceiling by ef=40 and
       all further ef budget only buys graph-exploration recall,
   (b) the exact-quantized harness uses the same approximate codes
       as grouped and so its ceiling *is* what grouped saturates to.

   The scalar sweep at 50k shows `exact-quantized Recall@10 = 0.8560`
   column across all ef rows (tables at lines 177-184), suggesting
   the ceiling is corpus-wide, not per-query, and is the same
   exact-quantized value for both indexes. So (b) is the explanation:
   both the grouped and scalar sweeps use the same
   `exact-quantized` computation, which happens to coincide with
   grouped's ef=40 result. The scalar ef=200 (0.8940) shows exact
   rerank can exceed quantized because scalar stores exact payload —
   wait, that's suspicious. Actually scalar exact-quantized ceiling
   is also 0.8560, yet scalar recall@10 reaches 0.8940 at ef=200.
   That means "exact-quantized" here is a *lower bound* proxy
   (ground-truth against scalar-quantized codes), not a ceiling on
   what the index can return.

   This isn't a bug in the packet — it's a reading hazard. "Recall@10
   of 0.8940 vs exact-quantized 0.8560" means the scalar index
   returns *more* true top-10 than the scalar-quantized exact
   computation identifies. That happens when the exact-quantized
   baseline itself is lossy, which it is (it uses the same
   quantization as the index under test). The "exact-quantized"
   column is useful as a comparator but not as a ceiling. Worth a
   request-level note so future readers don't mistake it for a
   ceiling.

### Observation

The keep/pivot call is right. Don't widen the SQL benching surface
until 50k closes some of the recall gap. Specifically: before the
next measurement batch, a window-width sweep is the single highest-
signal experiment you can run. If window=4 is too narrow, widening
it is the cheapest fix you can make, and the result tells you
whether the issue is *only* rerank prefix or whether the approximate
order itself needs work.

I'd also pull one small datum the packet didn't record: the grouped
direct-harness timing at 50k ef=40 (1.60ms) vs 10k ef=40 (0.87ms).
The 10k→50k grouped slowdown is ~1.85x at ef=40, but ~1.72x at
ef=200. For scalar it's ~1.51x at ef=40 and ~0.55x(!) at ef=200 —
the latter is nonsense unless the 10k ef=200 run was cold and 50k
ef=200 was warm. More evidence that the 50k sweep has cache-state
noise in it.

### What this packet earns

It earns the right to:
- keep the grouped-v2 runtime lane open
- defer the full 50k SQL benchmark matrix
- spend the next runtime slice on pipeline tuning (window width,
  explicit binary-prefilter integration, per-query setup cost)

It does NOT earn:
- a gate-lift discussion
- 50k latency claims — the numbers have cold-cache noise and 10
  queries of variance
- a "50k is at parity" headline — grouped is below scalar on recall
  at every ef point measured

Next-slice direction agrees with the request's "Next Step" list —
inspect window=4 width, wire binary-prefilter properly, re-measure.

### Measurement gap still open

Corpus-scale recall is no longer an open gap at 10k — it's answered
("directionally good"). It's still open at 50k in the specific sense
that grouped is currently *worse*, which is a negative result, not
missing data. Negative results are still evidence. The next packet
should either close the 50k gap or explicitly reframe the target
operating point (e.g., "grouped is for latency-constrained
k=1/k=5 reads, not 50k k=10"), not add more benchmarks at the same
operating point.
