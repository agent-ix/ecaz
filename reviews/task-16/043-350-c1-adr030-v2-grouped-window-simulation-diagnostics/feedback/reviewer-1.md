## Feedback: ADR-030 v2 Grouped Window Simulation Diagnostics

Read `debug_grouped_scan_windowed_rows_from_comparison_rows` at
`scan_debug.rs:1759` and `debug_grouped_window_size` at `scan_debug.rs:1691`.

### What's right

- Simulating the rerank window before wiring it into live scan is the correct
  order of operations. Avoids a two-step change ("introduce window behavior
  and redefine what 'emitted order' means") at the same time. Packets 346-349
  built a diagnostic surface that treats emitted order as approximate order;
  changing that mid-stream would invalidate those diagnostics.
- `debug_grouped_window_size` validates `window_size > 0` with a clear error.
  Zero-size windows would silently produce zero-row output under the current
  algorithm (outer `while output_rows.len() < rows.len()` would spin with
  empty buffer), so guarding at the boundary is right.
- Reuses `debug_grouped_scan_comparison_rows` from packet 348. No
  reimplementation of rank-derivation logic.
- Summary wrapper reports before/after metrics side by side. That's what
  the rerank-window selection actually needs — not the window's metrics
  alone, but the delta over the baseline.
- Shared `debug_grouped_scan_windowed_rows_from_comparison_rows` is pure (no
  pg_sys, no unsafe). Unit-testable end-to-end.

### Concerns

1. **Missing-comparison fallback uses approximate score.** Line 1779:
   `let left_exact = left_score.unwrap_or(left.2);` — if a row has no
   comparison score, the simulation falls back to the approximate score for
   window selection. Three options for what that should mean:

   - (A) Drop missing-comparison rows from simulation (can't rerank what
     can't be scored).
   - (B) Leave them in approximate order (current behavior — fall back to
     approx).
   - (C) Skip them in selection; treat as "cannot reorder."

   Current code does (B). Defensible, but it gives the simulation
   artificially good-looking metrics when comparisons are frequently
   missing — the missing rows appear to be "rerank-correct" because they're
   scored consistently at the approximate value. Two interactions worth
   naming:

   - Today packet 346's `grouped_candidate_rerank_comparison_score`
     pgrx-errors on mismatched cold payload, so "missing comparison" is
     effectively zero in live. If that behavior is softened (as I suggested
     in 346 feedback), this fallback starts mattering.
   - Packet 347's summary reports a `missing_comparison_count` metric.
     Paired with simulation, an operator reading "max rank shift after
     windowed rerank is 0" should know that reflects "the missing rows
     didn't get reordered," not "perfect rerank."

   Add a comment at line 1779 naming the fallback semantic explicitly, so
   a future reader knows this is a choice, not an oversight.

2. **Tail window shrinks.** Once the input is exhausted, the buffer drains
   without refill. For a query emitting 10 rows with `window_size = 4`,
   the last 3 emitted rows come from a 3-row, then 2-row, then 1-row
   buffer. That's standard sliding-window behavior, but it means
   simulation metrics slightly underestimate the effectiveness of a real
   window at the tail of output. Probably immaterial for k=10 queries;
   could matter for k=1000. Worth naming in a docstring.

3. **Tie-break.** Line 1783: `then_with(|| left.1.cmp(&right.1))` uses
   `left.1` which is `approx_rank` — stable tiebreak by original emission
   order. Good. But 4-bit grouped quantization produces ties frequently,
   so the tie-break is load-bearing. Worth a test that constructs an
   input where several rows have identical comparison scores and asserts
   the output preserves original emission order for the tied rows.

4. **`window_size == 1` is a no-op.** Buffer holds one row, always picks
   it, `windowed_rank == approx_rank`. Worth an explicit test asserting
   this invariant — it's a useful sanity check and catches a whole class
   of off-by-one simulation bugs.

5. **No upper bound on `window_size`.** `debug_grouped_window_size` only
   rejects non-positive. A window size exceeding the emitted row count
   effectively becomes "sort all rows by exact." That's the "infinite
   window" case and is meaningful — it tells the operator the best
   possible reordering from the emitted set. Not a bug but worth making
   sure the pg-test exercises this boundary (`window_size == emitted_count`
   and `window_size > emitted_count`).

### Observation

This is good staging discipline. Before the next packet flips live scan
behavior, the operator can SQL-drive the simulation, pick a concrete
window size backed by evidence, and know what it will do. The real
question is whether the next packet actually uses the simulation to
justify its window choice rather than defaulting to a fixed small number
like 4 or 8 "because that's where the window evidence summary already
points." Either way, preserve this diagnostic past the cutover — after
the live window lands, the simulation surface is what lets you A/B test
alternate windows without changing behavior.

### Measurement gap still open

No corpus-scale recall numbers yet. Simulation tells you what happens
*within* the emitted set; it cannot tell you whether the emitted set
itself is right. A gate-lift conversation still needs a corpus recall
test (grouped-v2 at ef_search=X returns Y% of the true top-10 vs scalar-
v1's Z%). Keep flagging this — 350 is the right packet but it still
operates on emitted-set diagnostics.
