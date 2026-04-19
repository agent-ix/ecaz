## Feedback: Persisted rerank-source measurement — ACCEPTED with questions

Verified against:

- commit `239238a` adding packet artifacts
- three cited warm-verified latency summaries + one recall TSV
- confirmed persisted-default lane via live runtime settings probe
  (`heap_f32 / default_heap_f32_with_rerank_source_column /
  source_raw`)

### What's right

- **Actually measures the persisted path.** The §4 caveat about
  `scripts/restart_adr030_scratch.sh` forcing `TQVECTOR_PQ_FASTSCAN_
  RERANK_MODE=heap_f32` is load-bearing: without the manual
  scratch-start workaround this packet would be re-measuring
  packet `430`'s env-override path under a persisted-looking
  label. Catching that explicitly, and documenting the workaround,
  is the right move.
- **Live runtime-settings probe quoted.** Not an inference — the
  debug probe result (`default_heap_f32_with_rerank_source_column
  / source_raw`) is included verbatim, so the measured lane's
  resolution is auditable.
- **Recall stays at the serious-lane exact-score point.**
  `0.9635` recall@10, `0` score error, `0` exact-gap queries,
  `0` below-exact queries — the productization did not regress
  correctness. Recall also reported at `@100`, spearman rho, and
  exact-quantized recall, which widens the evidence base.
- **Multiple runs reported, honestly.** The three-run spread
  (`4.649 / 5.386 / 6.361ms`) is not averaged into a hero number.
  The packet explicitly refuses to claim "persisted == env
  override" latency parity — that is the right call given the
  data.

### Concerns

1. **Noise is the dominant reading.** Min `4.65ms`, max `6.36ms`
   across three nominally-identical cells. That spread exceeds the
   full gap this arc has been chasing (`5.046 → 4.568ms` across
   all of `423 → 430`). A measurement packet that cannot separate
   signal from scratch-restart noise is not yet sufficient to
   license a "this is ready to ship" call, even if the readout is
   honest about it.
2. **Restart-helper fix is named but not landed.** Packet `431`
   should have shipped with a helper fix so packet `432` could use
   the standard path. Please land a short follow-on that teaches
   `restart_adr030_scratch.sh` to not force `RERANK_MODE` when the
   user requests a mode that matches the persisted default (or
   simply accept a `--no-rerank-mode-override` flag).
3. **No planner-path cross-check.** Packet `423` used one-index-
   per-table surfaces and verified warm SQL paths. This packet
   reuses those tables but doesn't show a planner `EXPLAIN` or
   timing-mode cross-check on the persisted-default cell. For a
   cell that is currently the lever-3 productization proof, that
   feels under-verified.

### Questions for coder-1

1. **Measurement noise origin.** Do you have a working theory for
   why run 2 (`6.361ms`, p99 `11.996ms`) diverged from runs 1 and
   3 by ~`1.7ms` on mean? Was the scratch postmaster restarted
   between runs, and did OS page cache state differ? If you re-run
   three cells with a pinned `DROP CACHE`/`pg_prewarm` protocol
   does the spread narrow? Without that, I cannot distinguish
   "persisted-path is equal to env-override path" from "persisted-
   path is slower but noise hides it."
2. **Stale-TID rollout story.** Packet `430` showed that backfilling
   the rerank source column leaves existing index entries pointing
   at stale heap TIDs until `REINDEX`. Packet `431` calls this
   unsolved follow-on. Before closing task 16, what's the intended
   shape — user-visible REINDEX requirement (documented),
   extension-emitted clearer error, or an automated helper? Does
   this belong inside task 16 or is it a followup task?
3. **Task 16 decision subtask — what's the lever-4/5 measurement
   plan?** The task doc still lists "Decide whether to pursue
   lever 4 (tiled LUT) and/or lever 5 (int8 LUT)" as open. The
   method here is to measure both directly, not to infer from the
   heap-fetch finding in `429`–`432` that the scoring-kernel
   levers are uninteresting. On which lane(s) do you plan to
   measure 4 and 5 — quantized default, heap-f32 recall-preserving,
   or both — and at which `(m, ef_search)` cells? Any adjacent
   theories (e.g., heap fetch/decode cost itself, rerank-source
   representation variants) that should also be measured before
   closing task 16?
4. **V3 vacuum concurrency.** Packet `428` wired vacuum for V3 at
   pass-unit granularity. Was `scripts/vacuum_concurrency_scratch.
   sh`'s 60-second concurrent INSERT + scan + VACUUM harness re-
   run against a V3 index? The review README flags that harness as
   the vacuum-safety proof; pass-level unit tests alone don't
   substitute.
5. **ADR-042 / ADR-043 sequencing.** Two PROPOSED ADRs (native
   HNSW build, `tqvec` column type) landed on this branch
   alongside the task-16 packets. ADR-043 in particular is a
   direct response to the `bytea`/`real[]` awkwardness surfaced in
   packets `430` / `431`. Is the intent for ADR-043 to be
   implemented as a condition of closing task 16, or is task 16
   closing against `rerank_source_column` and ADR-043 landing
   later as its own task?

### Arc-level read

Across packets `422 → 432`, the arc is well-executed as an
investigation:

- Baseline (`423`) correctly invalidated stale assumptions in the
  original task text.
- Levers 1–2 (`424`/`425`) are justified and landed policy-only.
- The decision point (`426`) was named honestly, not papered over.
- V3 (`427`/`428`) was split into dormant substrate + runtime
  wiring, which is the right review hygiene for a wire-format bump.
- V3's measured result (`429`) produced the non-obvious finding
  (cost is heap rerank) and `430` tested the natural next probe.
- `431` productized the best-measured lever.
- `432` honestly refuses to claim more than the noisy data supports.

That is the shape of a good iteration arc. The five questions above
are about closing out cleanly, not reopening any of the landed
levers.
