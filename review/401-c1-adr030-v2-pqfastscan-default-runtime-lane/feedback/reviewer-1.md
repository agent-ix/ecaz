## Feedback: PqFastScan Default Runtime Lane

Read `src/am/scan.rs` (defaults at :48–51, env overrides), the
layout-aware traversal-score fallback, the re-exported constants in
`src/am/mod.rs`, the effective-default reporting in
`tqhnsw_debug_pq_fastscan_runtime_settings` / `_adr030_runtime_settings`
in `src/lib.rs`, and the updated `scripts/restart_adr030_scratch.sh`.

### What's right

- **Defaults now reflect the proven operating point.**
  `PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW` is `64` and
  `PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME` is `"binary"`. Packets
  `361` / `362` already showed `0.910` and `0.936` Recall@10 on the
  `50k` canonical lane at that shape; shipping a first-class format with
  a strictly weaker env-less default would have been dishonest. This
  closes the "experimental-in-defaults" gap I flagged across several
  earlier packets.
- **Layout-aware traversal-score fallback is the right shape.**
  `PqFastScan` layouts without a persisted binary sidecar fall back to
  grouped-PQ traversal rather than blindly picking binary on a layout
  that can't score it. That keeps the default correct for legacy-shape
  indexes built before the binary sidecar bit was persisted, without
  giving up the win on the new build.
- **Debug helpers now surface effective defaults, not just env
  overrides.** The old shape (only show what was explicitly set)
  actively hid the operational truth; an operator could read the
  helper and still not know what window they were actually getting.
  This is a small change with real debuggability value.
- **Scratch restart helper aligned with code defaults.** Previously the
  scratch wrapper itself injected the stronger lane via env; after
  this packet it can start empty-flagged and still match the code.
  That removes one class of "benchmark numbers depend on which
  wrapper I used" footgun.
- **Tuning surface preserved.** The canonical
  `TQVECTOR_PQ_FASTSCAN_*` envs (and legacy
  `TQVECTOR_EXPERIMENTAL_ADR030_V2_*` fallbacks) still work. The
  packet correctly frames "promote defaults, don't remove knobs."

### Concerns

1. **No benchmark rerun on the new binary.** The packet correctly scopes
   out "fresh rerun" but the merge-readiness story for task 15 depends
   on showing the default-built binary on the canonical real-corpus
   lane producing the same numbers as the env-tuned runs from `361` /
   `362`. That rerun is the immediately-next execution step and should
   be captured in a dedicated packet — without it, the claim "proven
   operating point is the default" rests on transitivity rather than
   observation.
2. **Traversal-score fallback is implicit.** An operator whose
   `PqFastScan` index lacks the binary sidecar will silently get
   grouped-PQ traversal at `window=64` rather than binary. That is
   the right behavior, but `tqhnsw_debug_pq_fastscan_runtime_settings`
   should state whether binary was selected or fallen back from, so
   operators investigating a recall shortfall can see that reason
   directly rather than having to inspect layout metadata themselves.
3. **`restart_adr030_scratch.sh` defaults re-duplicate constants.**
   The shell wrapper now encodes `window=64` / `grouped_score_mode=binary`
   literally. If the code defaults move again, these drift silently.
   A comment in the wrapper pointing at
   `PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW` /
   `PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME` would keep the
   dependency visible without introducing a dynamic read.
4. **Linker gap unchanged.** `cargo test` and `cargo pgrx test pg17`
   still blocked on the usual PostgreSQL symbol family; the new
   effective-default pg tests added to `src/lib.rs` are proven only
   by `cargo check --tests` + clippy locally. Same systemic caveat as
   the rest of the 378–400 arc.
5. **`cargo fmt --all` blocked by unrelated `src/quant/prod.rs`
   parse failure on `rng.gen()`.** Correctly flagged as out-of-scope,
   but this is now the second or third packet noting it — it will
   block the merge commit when fmt runs in CI. Worth a tiny separate
   slice on this branch to unblock `fmt` before landing.

### Observation

This is the right-shaped packet to precede the final task-15 execution
run. It makes the default-lane story honest without either removing
escape hatches or inventing new configuration. The one thing left that
could change the merge story is the actual recall/latency rerun on
this binary against the canonical `pq_fastscan` / `turboquant`
real-corpus lanes.
