# Task 29 Release Latency Refresh — Round 3 Review

Reviewer: opus-review
Branch: `task29-diskann-initial-tuning` @ `761f6823`
Scope: Round-3 merge sign-off after the release-mode latency
re-measurement (`11105`) and the new pgvectorscale head-to-head.
Companion to round-1 (`review/11099-...`) and round-2
(`review/11103-.../feedback.md`).

## Verdict

**Code quality and measurement methodology are merge-ready. Three
final perf items will be tackled on this branch before merge per
user direction**: the build heap-frontier release-mode A/B
(genuinely suspect, deserves a definitive answer), L=64 scan
latency parity with pgvectorscale (the cleanest constant-factor
gap left), and DiskANN build performance (12× behind pgvectorscale,
worth chasing). Tracked as Task 29d
(`plan/tasks/29d-diskann-pre-landing-perf-sweep.md`).

The round-2 blocker is closed, the release-mode numbers strengthen
the landing narrative meaningfully, and the pgvectorscale
head-to-head gives a defensible reference posture. 167/167 DiskANN
tests pass at HEAD, clippy clean (re-ran both). The branch is in
materially better shape than I expected after round 2 — what's
left is polish-on-already-good, not block-of-not-good.

## What changed since round 2

### The release-mode re-measurement (`11105`) confirmed predictions

Round-2 prediction: release-mode latency drops 5-10× from the
debug-mode numbers cited in `11098`/`11103`. Result:

| L | debug (11098) | release (11105) | speedup |
|---|---|---|---|
| 64 | 50.36 ms | 8.05 ms | 6.3× |
| 128 | 48.80 ms | 8.20 ms | 6.0× |
| 200 | 53.15 ms | 8.74 ms | 6.1× |
| 400 | 58.89 ms | 9.02 ms | 6.5× |
| 800 | 68.90 ms | 9.57 ms | 7.2× |

Consistent 6-7× speedup, exactly the right shape for SIMD codegen
+ inlining + bounds-check elision on a tight numerical loop.
Recall is unchanged across the install-mode swap, as expected
(recall is not affected by codegen).

This flips the framing vs `ec_hnsw`. From round-2:

> DiskANN is ~50% slower than HNSW at equivalent recall settings
> on this corpus.

That was wrong, and was wrong because of the debug-install
artifact. The actual release-mode story:

| | recall@10 | mean lat (L=200) | p99 | index size |
|---|---|---|---|---|
| ec_diskann (sidecar) | **0.9970** | **8.74 ms** | **11.5 ms** | **4.7 MiB** |
| ec_hnsw (reference) | 0.9700 | 35.25 ms | 49.1 ms | 13.0 MiB |

DiskANN now wins HNSW on **all four axes** at this operating
point: recall (+3pp), mean latency (4.0× faster), p99 (4.3×
faster), storage (-64%). The trade-off it pays is build time
(13× slower than HNSW). That's a legitimately strong landing
position, not the "different shape, defensible" framing I had in
round 2.

### pgvectorscale head-to-head (`11105`)

The comparison addresses round-2's pre-empt #4. Methodology is
sound:

- Same 10k corpus, same queries, same brute-force ground truth
  used for both engines.
- Matched build parameters: `num_neighbors=32`,
  `search_list_size=100`, `max_alpha=1.2` on pgvectorscale ↔
  `graph_degree=32`, `build_list_size=100`, `alpha=1.2` on ours.
- pgvectorscale `storage_layout=memory_optimized` (1-bit SBQ) —
  the apples-to-apples comparison vs our 1-bit binary sidecar.
- Same release-mode warm-cache state.
- New `ecaz compare vectorscale` CLI plumbs both engines through
  the same recall + latency machinery.

Results:

| L | ec_diskann recall / mean | pgvectorscale recall / mean |
|---|---|---|
| 64 | 0.9965 / 9.19 ms | 0.9960 / 3.56 ms |
| 128 | 0.9965 / 8.06 ms | 0.9990 / 5.84 ms |
| 200 | 0.9970 / 10.4 ms | 1.0000 / 8.85 ms |
| 400 | 0.9970 / 9.86 ms | 1.0000 / 16.2 ms |
| 800 | 0.9975 / 10.1 ms | 1.0000 / 31.2 ms |

Build: ec_diskann 70.678 s, pgvectorscale 5.82 s.
Storage: ec_diskann 4824 kB, pgvectorscale 5016 kB.

Per-axis interpretation:

- **Build**: 12× slower than pgvectorscale. This is the same gap
  as vs HNSW, and is genuinely the only axis where ec_diskann
  loses materially. Fine landing follow-up.
- **Storage**: parity (4824 kB vs 5016 kB). Both are 1-bit-per-dim
  prefilter codes plus graph; small index payload.
- **Recall**: pgvectorscale slightly higher at high L (1.0000 vs
  0.9975), parity at low L. See methodology note below for why.
- **Latency**: pgvectorscale faster at L=64 (~2.6×), parity at
  L=200, ec_diskann meaningfully faster at L=400+ (3× at L=800).
  The curves cross around L=200.

### One methodology note on the comparison (not a blocker)

The `ecaz compare vectorscale` CLI sweeps `list_size` for both
engines, but pgvectorscale's `query_rescore` defaults to the
sweep value while ec_diskann's `rerank_budget` stays fixed at the
reloption default of 64.

That's a reasonable harness choice — each engine uses its
"natural" rescore setting per list-size — but it's worth being
honest about what it implies:

- At L=800, pgvectorscale reranks 800 candidates exactly. We
  rerank at most 64. That asymmetry explains both why
  pgvectorscale hits 1.0 recall at high L (more candidates get
  the heap-fetch rerank pass) **and** why pgvectorscale's
  latency grows steeply at high L (12.5× more rerank work).
- At L=64, both engines are reranking essentially everything in
  the frontier, so the comparison is more apples-to-apples.
  ec_diskann's 9.19 ms vs pgvectorscale's 3.56 ms there is the
  cleanest single number for "scan + rerank constant factors".

If reviewers ask, the right framing is: "pgvectorscale exposes a
larger rerank window by default; ec_diskann exposes a smaller
fixed window. Both choices are reasonable defaults; the sweep
table reflects each engine's natural operating point per
list-size value." The L=64 row is the apples-to-apples constant-
factor comparison.

Worth a one-line sentence in the merge discussion or in any
follow-up perf packet, but this isn't a methodology bug — it's a
harness design choice that is correctly documented in the CLI
arg help (`--vectorscale-query-rescore` is a flag).

### A second pre-empt that landed

The build heap-frontier experiment (round-2 item 2) was not
re-tested in release mode. Coder-1 shipped without it. That's the
right call given the time constraint — the active-mask prune
(`11104`) already closed the only release-mode build optimization
that mattered for this slice. The heap-frontier-build question
remains open as a single-experiment follow-up; nothing in the
landing decision turns on it.

## Code review of the new bits

### `crates/ecaz-cli/src/commands/compare/vectorscale.rs` (509 LOC)

A new comparison harness that builds, populates, and queries a
pgvectorscale-side `vector(1536)` sidecar table on the same
corpus. Reuses ecaz's existing `brute_force_top_k`, `recall_at_k`,
`ndcg_at_k`, and latency `summarize` helpers, so the comparison
metrics match what `ecaz bench recall` and `ecaz bench latency`
emit for ec_diskann. Right pattern.

A few small notes, none blocking:

- The validation block at lines 60-75 catches the obvious bad
  inputs. `vectorscale_num_neighbors <= 10` is an interesting
  lower bound — pgvectorscale's documented minimum is 10, so
  rejecting `<= 10` is one off. Should probably be `< 10`. Tiny
  cosmetic bug; doesn't affect any landing measurement because
  the default is 32.
- `rebuild = false` is the safer default; the rerun in the packet
  used `--rebuild` to force a clean comparison. Good.
- `prefer_ordered_ann_path(&client)` (line 160) is a nice touch
  — without it, pgvectorscale might fall to seqscan for tiny
  corpora and confuse the comparison.

The CLI is genuinely useful infrastructure. Keep it in main even
after Task 29 lands; future perf tuning will reuse it.

### Round-2 cleanup items

All 167 DiskANN tests pass at HEAD. `cargo clippy
--all-targets --no-default-features --features pg18 -- -D warnings`
clean. No regressions from the round-2 verdict.

## Risks and follow-ups

### Active risks (none are landing blockers)

1. **Build performance gap vs pgvectorscale (12×) and HNSW (13×).**
   Now the sole open performance question. Coder-1's `11102`
   profiling already isolated it to Vamana graph construction
   inside pass 1 (greedy search + robust prune + backlink
   repair). The `11104` active-mask cleanup got 11% there;
   another 4-5× would put it in the same range as pgvectorscale.
   Likely candidates per the 29c plan's deferred Phase 2 list:
   in-build heap-frontier (release-mode A/B), distance caching,
   build-time tuple cache. None are required for landing — the
   trade-off (build cost vs query / storage / recall) is
   defensible — but a future Task 29d or Task 29e for build perf
   is the natural next investment.

2. **pgvectorscale's L=64 latency edge (2.6× faster).** Real, and
   the cleanest constant-factor signal in the comparison. Likely
   sources include: their per-page overhead (more compact node
   layout?), frontier ops, or query setup. Worth a profile pass
   but no urgency — at L=200 ec_diskann is only 1.2× behind, and
   at L=400+ ec_diskann is faster.

3. **The rerank-asymmetry in the comparison harness.** Documented
   above. Not a methodology bug; just be ready to explain it if
   reviewers ask.

### Items addressed since round 2

- ~~Release-mode latency re-measurement~~ — DONE in `11105`.
- ~~pgvectorscale comparison~~ — DONE in `11105` with a new
  reusable CLI.
- ~~Heap-frontier build A/B in release mode~~ — explicitly
  skipped, will not block. Reasonable.

### Items addressed earlier and still standing

- 29b vacuum prefilter consistency: addressed.
- 29b GUC end-state + pgrx test: addressed.
- 29b SIMD codegen verification: addressed.
- 29c structured ambuild timing: addressed and shipped as
  production observability.
- 29c active-mask `robust_prune`: addressed.
- 29c debug-vs-release install hygiene: addressed.

## Path to landing

Per user direction: tackle all three remaining perf items on this
branch before merge. Tracked in Task 29d
(`plan/tasks/29d-diskann-pre-landing-perf-sweep.md`):

1. **29d-1 — Build heap-frontier release-mode A/B (½ day).**
   Cherry-pick `d2e0e9fc` + `36f0c3d5` onto current HEAD, install
   release, re-run the same isolated DROP+CREATE measurement, and
   compare to the 70.678 s active-mask baseline. Decision matrix
   in the plan: ≥ 5% win → re-land, ±5% → leave reverted with
   "no benefit", ≥ 5% loss → asymmetry is real. Resolves a
   genuinely suspect deferred question — same data structure shape
   as the scan-side win became a debug-mode regression on the
   build side, and that asymmetry deserves a definitive answer
   one way or the other.
2. **29d-2 — L=64 scan latency parity (1–2 days).** Profile the
   2.6× gap (9.19 ms vs pgvectorscale's 3.56 ms at L=64 — the
   cleanest constant-factor signal in the comparison). If a single
   dominant contributor emerges, attack it; otherwise document
   the breakdown. Stop condition: L=64 mean below 6 ms (within
   1.7× of pgvectorscale) or explicit residual.
3. **29d-3 — Build performance attack (1–3 weeks).** The big one.
   Profile pass-1 graph construction (already known to be 67.6 s
   of the 70.7 s total), then attack ranked candidates: SIMD the
   build distance, cache decoded tuples during build, distance
   reuse between greedy and prune, etc. Stop at within 3× of the
   strongest reference (≤ 17.5 s given pgvectorscale's 5.82 s) or
   when the next attack would take >5 days for <15% of remaining
   cost. Document either way; merge readiness keys off whichever
   stop fires first.
4. **Final 29d landing-readiness packet.** Refresh the full sweep
   (recall + latency + storage + build) on the post-29d head;
   round-4 review feedback; sign-off; merge.

The work that landed since round 2 is clean execution: the
release-mode re-measurement was the single ask, it confirmed the
predicted speedup, and the bonus pgvectorscale comparison gave the
landing decision a real reference anchor instead of just an
internal one.

After 29d lands, the merge story is: a real DiskANN access method
that beats HNSW on every query-time axis at competitive storage
and recall, with build cost reduced from 12× to a documented
ratio against the strongest reference, and three deliberately-
deferred perf questions all answered rather than left as
follow-up debt.
