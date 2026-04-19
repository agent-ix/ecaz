## Feedback: TurboQuant persisted-source supported measurement — ACCEPTED as a measurement packet, modest win, **n=1**

Verified against:

- packet `439`'s persisted `rerank_source_column` surface
- same-index ALTER SET/RESET methodology (same table, same index,
  same install, only the reloption flips)
- four summaries in `tmp/`:
  - `task16-turboquant-persisted-source-source-current.summary`
  - `task16-turboquant-persisted-source-source-current-q200.summary`
  - `task16-turboquant-persisted-source-source_raw-current.summary`
  - `task16-turboquant-persisted-source-source_raw-current-q200.summary`
- rerank micro-profiles and the recall summary TSV under
  `tmp/real_corpus_runs/20260419T005004Z_...`

### What's right

- **The stale-install catch is a real quality signal.** The
  packet names that the first scratch run produced flat
  `~26.96ms` and then correctly identifies the cause as a stale
  `tqvector.so` from before the reinstall, and *does not use
  that run*. That is the right way to surface a measurement
  hazard — the failed run is reported, not hidden, and the
  authoritative run starts only after
  `install_adr030_pg17_pg_test.sh`. The "proof not assumptions"
  methodology was on display here: the flat number was suspicious,
  the cause was chased, and the real measurement was repeated.
- **Methodology is genuinely apples-to-apples.** Same install,
  same table, same index name, same backend settings, same
  serious lane — only the reloption flips. This is the tightest
  shape of comparison possible for a persisted-reloption switch
  and it's the right response to packet `431`'s "only env, not
  persisted" gap.
- **The win is in the bucket the earlier packets predicted.** The
  rerank micro-profile shows `heap decode` dropping from `1488us`
  to `1386us` (`-102us`), fetch dropping from `107us` to `98us`,
  dot unchanged. That matches packet `429`'s stage profile and
  packet `430`'s same-table readout — the rerank bucket is where
  `source_raw` helps, not the scorer.
- **Recall is pinned.** `graph_recall_at_10 = 0.9629` and
  `mean_abs_score_error = 0` on the persisted-`source_raw` run
  matches the recall-preserving serious lane established in
  packets `429` / `430`. Packet `441` then hits the same number
  exactly, which corroborates this.
- **Honest top-line framing.** The packet does not claim the
  `-4.33%` recovers task 16. It explicitly says the serious lane
  is still `~4.8ms`, well above the quantized lane, and points
  forward to heap storage layout as the real limiter — which is
  exactly what `441` then proves out.

### Concerns

1. **q50 run is unusable on this delta size.** Both means landed
   at `4.746ms` at q50, i.e. indistinguishable. At q200 the delta
   is `-0.219ms`. That confirms the packet's own diagnosis that
   q50 is too noisy, but it also means the whole verdict depends
   on **one q200 cell per side**, n=1. The rerank micro-profile
   corroborates the direction, but for a `-4.33%` claim to
   carry forward as the "supported-path win is real" headline
   it would benefit from at least a 2× rerun at q200, or a
   reported spread.
2. **No cold-cache run.** The measurement is `warm-after-prime3`.
   That's the right primary surface for micro-level scorer
   comparisons, but the actual runtime win of an inline
   `source_raw` (later in packet `441`) is *especially*
   sensitive to cache behavior, so a cold-cache companion run
   at q200 would help future-you know whether the `-4.33%` is
   robust under warm-up variance or is an artifact of the prime
   passes. Not a blocker for this packet; the prime-pass shape
   is consistent with packets `429` / `430` / `437`.
3. **Same-index ALTER SET/RESET methodology depends on the
   reloption actually flipping the runtime, not just the
   catalog.** Packet `439` proves the DDL round-trips and packet
   `440` relies on that. A `pg_test` of the ALTER cycle (as noted
   in the `439` feedback) would lock this in — right now the
   measurement trusts the DDL is wired, which is fine once but
   shouldn't be a durable assumption.
4. **"About `4.3%`" is inside the noise envelope from packet
   `432`** (`4.65 – 6.36ms` spread across early cells). The
   rerank micro-profile is what makes this more than noise,
   since `heap decode` moved in the expected direction. But if
   only the top-line q200 number were in evidence, this packet
   would be "noise-adjacent." Worth naming explicitly that the
   *micro-profile* is doing the load-bearing work here, not the
   top-line delta alone.

### Questions for coder-1

1. **Does q200 ×3 reproduce the `-4.33%`?** With one run per
   side, we cannot separate a `4%` supported-path win from
   restart/prime-pass noise with high confidence. Worth one
   more pair (either q200 ×2 or q500 ×1) before packet `441`'s
   much larger `-35.16%` result is treated as the whole story.
2. **What happens if `rerank_source_column` points at a column
   that was wiped from the heap between SET and query?** Not a
   correctness question for this measurement, but packet `440`
   relies on the reloption being dynamically re-resolved at
   scan time — worth confirming that path is covered or
   explicitly queued.
3. **Does the `install_adr030_pg17_pg_test.sh` → `restart` →
   measurement sequence have a guard that warns when the
   shared library on disk and the `.so` the backend loaded
   don't match?** This catch was manual ("numbers looked flat,
   so the install state was investigated"). A script-level
   version check would convert a manual debugging step into an
   automatic safety belt and make future measurement runs
   harder to get wrong.

### Call

Accepted as the supported-path measurement for the persisted
`rerank_source_column` reloption on TurboQuant. The `-4.33%` is
real in direction (micro-profile corroborates) but small enough
that a rerun at q200 would tighten the confidence interval.
Packet `441` then makes this measurement matter — it is what
motivates the storage-layout lever that actually moves the
serious lane by `-35%`.
