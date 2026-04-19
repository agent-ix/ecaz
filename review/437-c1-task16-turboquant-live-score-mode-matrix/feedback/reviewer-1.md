## Feedback: TurboQuant live score-mode matrix — ACCEPTED, with a real bug surfaced

Verified against:

- commit `e49e835` (measurement-only packet on top of `572dd53`)
- eight latency summaries and eight recall summaries in
  `tmp/` and `tmp/real_corpus_runs/`
- isolated matrix surface `tqhnsw_real_50k_task16_lutcmp_*`
- stated `./scripts/vacuum_concurrency_scratch.sh --socket-dir
  /home/peter/.pgrx --duration 60` failure with
  `unexpected tqhnsw scan result count: 0`

### What's right

- **Full 4×2 matrix against the question the user asked.** Four
  scorer modes × two rerank lanes at `m=16, ef=128`, warm-verified
  SQL, all against the same rebuilt index. Exactly the
  apples-to-apples shape required to distinguish scorer effects
  from rebuild effects.
- **Live data overturned the offline verdict.** Packet `433` said
  lever 4 was not justified. Running on the real scan path showed
  `full_lut` is `-16.26%` and `tiled_lut` is `-16.55%` against
  exact on the quantized lane, with recall preserved. This is the
  kind of reversal that only measurement produces — exactly why
  the "proof not assumptions" method exists.
- **Honest verdict on the serious lane.** On `heap_f32` the deltas
  are `-1.48% / -1.74% / +2.97%`. The packet does not cherry-pick
  the quantized-lane win and claim task 16 is closed. It names
  that the serious-lane bottleneck is still not the scorer.
- **V3 vacuum-concurrency rerun actually ran, and the result is
  reported.** The packet does not hide a failing run. The scan
  workers hitting `unexpected tqhnsw scan result count: 0` is
  called out explicitly as a separate follow-on issue. This is
  the correct response to the packet-`428` feedback ask — data,
  even when the data is inconvenient.
- **Recall matches across scorers (within int8's measured drift).**
  `full_lut` and `tiled_lut` produce bit-identical `mean_abs_score
  _error` (`0.006030937`), matching exact. That is consistent with
  packet `433`'s exactness tests and lets the scorer comparison
  stand on latency without a recall confound.

### Concerns

1. **V3 vacuum concurrency is broken, not just "unverified."**
   This is no longer the close-out question from packet `428` —
   it is a live concurrency bug discovered by running the
   requested harness. `unexpected tqhnsw scan result count: 0` is
   a scan returning empty under concurrent vacuum, which for a
   serving index is a correctness regression against the A6
   vacuum contract documented in the review README.

   **Please open a tracked task** (either a new `coder2` slot or
   an explicit task-16 blocker) rather than letting this sit as a
   prose bullet in a measurement packet. I would also argue this
   blocks any merge that relies on V3 being the default
   TurboQuant writer (per packet `428`, which is already the
   case).
2. **n=1 per cell, eight cells.** As with the earlier serious-lane
   measurements, single-run cells. The `heap_f32` deltas (`-1.48%`,
   `-1.74%`, `+2.97%`) are well within the packet-`432` noise
   envelope (`4.65 – 6.36ms` spread). On the quantized lane the
   `-16%` lever-4 signal is large enough to survive that
   envelope. On the serious lane, the sub-3% differences are not
   individually distinguishable from restart noise without reruns.
   Please rerun the four heap-f32 cells 2–3× before treating that
   table as a verdict.
3. **`full_lut` vs `tiled_lut` verdict is close to a coin flip.**
   On the quantized lane the live deltas are `2.333ms` vs
   `2.325ms`. §4 correctly says tiled is "not justified over full
   from this cell alone" — that is the right call. Worth
   carrying forward explicitly: if lever 4 lands, the default is
   full, not tiled, unless a later cell shows tile-size tuning
   wins.
4. **No planner `EXPLAIN` attached.** The matrix uses verified SQL
   latency helpers; packet `423`/`426` also used planner-verified
   surfaces, but this packet doesn't show the planner path
   actually selected the tqhnsw index for every cell. Worth one
   `EXPLAIN (ANALYZE, BUFFERS)` capture on the baseline exact
   cell as proof.
5. **Quantized-lane recall identical across `exact`, `full_lut`,
   `tiled_lut`.** That is correct — they are bit-identical
   scorers — but worth naming in the readout that recall matching
   is a *correctness* assertion on current head, not a happy
   accident. If a future lever changes the prefilter-to-rerank
   handoff, that identity might not hold, and packet `437`'s
   table would need re-running.

### Questions for coder-1

1. **Vacuum concurrency failure — next step?** Is the plan to
   file a new task and continue task 16 closeout, or does task 16
   hold on fixing V3 concurrency first? The packet calls it a
   "separate follow-on" — that needs an owner + date, not just a
   bullet.
2. **Does the `unexpected scan result count: 0` reproduce
   deterministically?** Or was it intermittent during the
   60-second run? Either answer changes the debugging approach.
3. **On the quantized lane, does `full_lut` actually win at
   higher ef_search?** The `-16%` delta on the quantized lane is
   the first scorer-lever result large enough to justify a real
   runtime change, but only one cell has been measured. Before
   this becomes a persisted default, please run the same matrix
   at `ef_search = 64 / 128 / 256` so the lever-4 decision isn't
   based on one point.
4. **Is `int8_approx`'s `+2.97%` regression on the heap-f32 lane
   consistent across reruns?** If yes, int8_approx is not just
   "no win on serious lane" but actively worse there, which
   matters for any future default flip. Worth confirming with at
   least two more runs.
5. **Rerank cost dominates on the heap-f32 lane (per packet
   `429`'s stage profile — `~1.22ms` rerank vs traversal).** With
   four scorer variants now live-measured, do you have a planned
   follow-on that targets the rerank bucket directly (heap
   fetch/decode) rather than the scorer? That is where the
   packet's own §3 points, and it is currently untouched by any
   landed lever.

### Call

Accepted as the lever-4/lever-5 live-runtime decision cell. Two
substantive outputs:

- **lever 4 is real on the quantized lane** — the opposite of what
  packet `433` inferred offline, and the most important reversal
  in the whole arc. The measurement method paid off here.
- **V3 vacuum concurrency is broken** — discovered by actually
  running the harness requested in packet `428` feedback. This
  is the most important finding in the packet and deserves a
  tracked task, not a closing bullet.

Pending the n=1 rerun concerns, the matrix is the correct decision
cell for the scorer question; the concurrency bug is blocker-shaped
and should be lifted out of this packet into its own tracking.
