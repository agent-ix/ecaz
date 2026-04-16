## Feedback: ADR-030 v2 Binary Window-64 Operating Point

Read the `ADR030_GROUPED_V2_MAX_LIVE_RERANK_WINDOW` bump from `16` to
`64` at `scan.rs:25`, the launcher override surface changes in
`scripts/bench_sql_latency.sh`, and the isolated grouped-only
measurement path.

### What's right

- **Raising the cap from 16 to 64 is justified by evidence.** The
  emitted-set window diagnostics from packet 354 showed recall still
  climbing at simulated `window=16` (Spearman 0.97 vs 0.87 at 8);
  this packet extends the cap so live runtime can actually reach
  that point. The direct frontier numbers at lines 156-163 confirm
  the extra width buys real recall (window=32 ef=128: 0.846;
  window=64 ef=128: 0.860).
- **Max cap via a compile-time const is still the right shape.** The
  `[BufferedGroupedScanResult; ADR030_GROUPED_V2_MAX_LIVE_RERANK_WINDOW]`
  array sizing pattern from 353 still holds — the buffer is sized
  to the new max 64 and active width is still a `u8` field. Memory
  footprint grows from 16 × struct to 64 × struct per scan opaque,
  which on a `Copy` struct of `{element_tid, approx_score,
  approx_rank_base, comparison_score, heap_tids}` is on the order
  of a kilobyte extra per scan. Fine.
- **Launcher override surface is narrow and well-scoped.**
  `--corpus-table`, `--query-table`, `--index-name` override the
  prefix-derived names, but `--prefix` stays the anchor. Exactly
  one `--m` required when `--index-name` is passed. That's the right
  pattern — the prefix is still the test-identity anchor, overrides
  are escape hatches, and the single-m constraint prevents ambiguous
  "which index does this name correspond to" lookups.
- **Regression test added for the override path.** Python
  `test_bench_sql_latency_verified.py` with an explicit grouped-style
  override. Prevents silent regression when the launcher gets further
  changes.
- **Honest readout of the canonical-shared-table planner problem.**
  Lines 217-226 document that the planner still chooses
  `tqhnsw_real_50k_m8_idx` even when the launcher is pointed at
  `tqhnsw_real_50k_grouped_m8_idx`. The launcher aborts before
  timing — that's the correct fail-closed behavior; better to abort
  than silently time the wrong index.
- **Isolated grouped-only fallback proves the override path
  end-to-end.** When the shared-table planner won't pick grouped,
  building an isolated grouped-only corpus and pointing the launcher
  at it gives a clean planner lane with no ambiguity. Not the
  operating point we ultimately want, but it unblocks the
  measurement surface.

### Concerns

1. **Linker-block continues.** Same local environment issue from
   359 — `cargo test` and `cargo pgrx test pg17` don't run. At this
   point it's been three packets (359, 360, 361 in the upcoming
   review) where the required checkpoints haven't run. This is
   accumulating risk: each of those packets is claiming pg test
   coverage (including `grouped_live_rerank_window_32_env_matches_simulation`
   from 360) that hasn't executed in the reported checkpoint. A
   deliberate fix-the-linker packet would be worth its weight now.

2. **The isolated-table numbers are strictly better than the
   canonical-table grouped numbers at the same configuration (lines
   252-260).** The packet explicitly warns against treating the
   isolated surface as the canonical operating point (good), but
   doesn't investigate *why* the isolated build is stronger. The
   observation — "different build surface, materially stronger
   recall" — is a planted flag for 361 to pick up, and 361 does
   indeed close the loop on this: the difference was graph-build
   non-determinism. Worth naming that suspicion here too:
   "surface stronger than canonical despite identical row order" is
   a strong signal that build reproducibility is not what it should
   be.

3. **Direct-harness frontier before rebuild (lines 151-163) was
   measured on an old canonical grouped index.** The packet caveat
   at line 165-167 is right ("the canonical grouped index had not
   been freshly rebuilt on the current scratch install yet"), but
   "before rebuild" and "after rebuild" numbers are in the same
   packet without a clear separator on the intended canonical
   result. Readers might quote the before-rebuild line by accident.
   Minor nit — the after-rebuild table at 175-184 is correctly
   flagged as canonical, so the mistake would be mine as a reader,
   not the packet's as a writer. Still, numbering the tables
   explicitly (T1, T2, ...) would remove the ambiguity.

4. **Same-recall comparison at ef=40 (grouped window=64) vs
   scalar is the headline.** Line 202: grouped `window=64, ef=64`
   reaches 0.860 in 1.007ms; scalar needs ef=40 at 1.398ms for the
   same 0.860. That's a 39% latency win at same recall. But the
   scalar lane at ef=40 isn't quite at the exact-quantized ceiling
   (0.860 is exactly the ceiling per line 190), so this is really
   "grouped 0.860 at ef=64 vs scalar 0.860 at ef=40 ceiling-pinned."
   Headlining this as a same-recall win is fair, but the context
   is that both are saturated at the quantization ceiling. Worth
   naming.

5. **Grouped's recall ceiling at 0.874 still trails scalar's 0.898.**
   Line 204: "grouped does not win at higher-recall operating
   points." That's the honest frame. If a user wants Recall@10 >
   0.874 on this corpus, grouped-v2 binary-window=64 can't deliver
   it — no amount of ef widening closes the gap within this packet's
   data. That becomes a product conversation: is grouped-v2's
   ceiling acceptable for the latency-first operating point? Packet
   361's deterministic-build fix partially moves this ceiling up,
   so the conversation is delayed, not resolved.

### Observation

This packet is the last in the "stage runtime experiments" arc
before 361 surfaces the build-lottery surprise. Read in retrospect,
the hints were all here:

- isolated grouped-only builds outperformed canonical shared-table
  grouped builds despite identical row order
- direct-harness vs SQL-launcher frontier stories were inconsistent
  enough to be suspicious

361 converts those hints into a concrete finding (non-deterministic
graph build). This packet did the right thing by advancing the
measurement surface and flagging the build-surface anomaly rather
than burying it. That's how hints get followed up.

### Measurement gap still open

- shared-canonical-table planner lane is open (planner prefers
  scalar). Packet 360's fix for the launcher side is right; the
  remaining piece is either a planner-side change or removing the
  scalar sibling from the canonical test schema.
- pg test coverage for `grouped_live_rerank_window_32_env_matches_simulation`
  is claimed passing but didn't run in the required checkpoint. See
  concern #1.
- size and external-baseline comparison with pgvector is open. That
  lands in 363.
