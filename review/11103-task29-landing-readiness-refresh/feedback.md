# Task 29 Landing Readiness Refresh — Round 2 Review

Reviewer: opus-review
Branch: `task29-diskann-initial-tuning` @ `4ff2b289`
Scope: Round-2 merge-readiness pass after 29b (`11100`) and 29c
(`11101`–`11104`) landed on branch. Companion to the round-1 review
in `review/11099-task29-diskann-landing-readiness/feedback.md`.

## Verdict

**One measurement gap stands between the branch and merge.** Code
quality is solid, the algorithmic changes are sound, the discovered
debug-vs-release artifact was caught honestly, and the release-mode
build numbers land at a defensible ratio vs the HNSW reference. But
**the latency numbers being cited for the landing decision were
captured with the same debug-installed extension that caused the
build-time confusion**, and have not been re-measured in release
mode. That re-measurement is the last thing 29 needs before merge.

I'd land it after one more packet that reruns the L=64..800 latency
sweep with `cargo pgrx install --release` on the same isolated
real-10k corpus.

## What's good

### 29b — vacuum prefilter consistency (`11100`, commit `95fef9ac`)

Implementation matches the plan. The `PreparedPrefilter` enum +
`prepare_prefilter` helper at `routine.rs:1205-1311` is exactly the
shape that prevents drift between `amrescan` and
`plan_vacuum_fill_candidates_for_target`. Both call sites collapse
to `prefilter.score(tuple)` inside the `greedy_descent_with` /
`vamana_scan_with` closures — there is now no per-site path that
could regress independently.

Recall measurement is the right shape: pre-vacuum 0.9970, kill 5%
of rows, post-vacuum live-row recall 0.9975. Recall-neutral as
expected, which is the consistency invariant the task wanted.

SIMD codegen check is honest: `cargo asm` not available locally so
they used `cargo rustc --emit=asm` and verified AVX2
`vpxor`/`vpshufb`/`vpsadbw`/`vpaddd` + tail `popcntq` directly.
Documented as a packet artifact. No rewrite needed.

GUC end-state landed: doc string updated to production rollback
intent, `pg_test_ec_diskann_prefilter_kind_override_switches_prefilter`
added. (Caveat below.) 167/167 tests pass, clippy clean — re-ran
both at HEAD.

### 29c — build performance + the debug-vs-release discovery (`11101`–`11104`)

The honest-mistake catch is the most important thing to land here.
Packet `11102` reran the same head and corpus that produced `11101`'s
~497 s number, this time with `cargo pgrx install --release`, and
got 79.238 s — a 6.2× delta consistent with typical Rust
debug-vs-release overhead on tight numerical kernels. That number
is well within what bounds-check elision + inlining + SIMD codegen
can account for, so the finding is real, not a different
measurement bug.

The active-mask `robust_prune` rewrite at
`vamana.rs:266-296` is a clean win:

- I traced through it on a small example: candidate set sorted
  ascending; cursor advances only forward; pivot selection and
  α-dominance check both apply to the same set as the original
  `remove(0)` + `retain` shape; result order is identical.
- Replaces O(N) shift on every pivot pop and O(N) compaction on
  every prune-step with O(1) cursor advance + O(N) bit-test scan.
  At N≈100, R=32 that's roughly 2× fewer element ops plus much
  better cache behavior.
- Measured: 79.238 s → 70.678 s release-mode index-only build
  (10.8%), Vamana core graph 75.903 s → 67.571 s (11.0%), pass-1
  53.363 s → 46.832 s (13.9%). Index size unchanged (4,939,776 B).
  Isolated L=200 recall@10 unchanged (0.9970).

The structured ambuild phase timing from `b9eba667` is a real
asset. Bracketed counters for model training, payload derivation,
medoid selection, Vamana graph (pass 0/1 split), persistence,
overflow stage, codebook chain, page writes. These should stay
in main even after Task 29 lands — they're production-useful
observability, not just task-29 instrumentation.

### Reference comparison

`ec_hnsw` build on the same real-10k table with `m=32`,
`ef_construction=100` is 5.23 s, vs DiskANN's release-mode 70.678 s.
That's a 13.5× gap on build time, in DiskANN's worst direction —
which is the expected trade-off. Storage goes the other way: 4.7
MiB DiskANN vs 14 MiB HNSW (3× smaller). Recall: 0.997 vs 0.97.

This is "different shape, defensible" rather than "DiskANN worse on
everything." It's a credible landing posture provided the latency
numbers also hold up in release mode (see below).

## What still needs to happen

### 1. Re-measure latency in release mode (LANDING BLOCKER)

**This is the only blocker I see.** The latency numbers cited in
`11103`'s landing summary (L=200 mean 58.5 ms, L=800 mean 67.7 ms,
etc.) come from packets `11097` and `11098`. The validation steps
in those packets say:

> After the pg_test run, the normal PG18 extension build was
> reinstalled and the local PG18 server was restarted.

No `--release` flag mentioned. By the same reasoning that 11102
correctly identified for builds, those reinstalls produced a
debug-mode extension binary. Every query through that binary —
including the latency sweep — runs with no SIMD, no inlining,
bounds checks intact, generic monomorphization paying full cost.

The plausibility check: at L=200 the cited mean is 58 ms. In a
release-mode binary, the per-visit cost is dominated by ~50 µs
page fault + ~30 ns Hamming popcount + heap fetch for top-K. For
200 visits + 64 reranks that's roughly ~10 ms warm-cache. The
68-ms-at-L=800 cited number similarly looks like debug-mode
single-digit-µs-per-popcount territory rather than release-mode
sub-µs.

There are two possible outcomes:

- **(Likely) Release-mode latency is meaningfully lower.** Maybe
  ~10 ms at L=200, ~15 ms at L=800. That would put DiskANN
  *faster* than HNSW (33 ms p50) at materially better recall.
  Strengthens the landing narrative.
- **(Unlikely) Release-mode latency is similar to the cited
  numbers.** Then the cited numbers were already release somehow,
  and nothing changes. Either way, you need to know.

Concrete ask: one new packet that does

```
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
# restart server
# rerun the same L=64/128/200/400/800 latency + recall sweep used in 11098
```

Single packet, half a day. Not an algorithmic change, just an
honest re-measurement of what should have been release all along.

### 2. Re-test the build heap-frontier experiment in release mode (recommended, not blocking)

Packet `11101` measured the build-side heap-frontier experiment as a
12% regression and reverted it. But that 12% regression was measured
in **debug mode** (it's the same packet that produced the ~497 s
baseline that turned out to be debug). Whether the heap-frontier
build change is a regression in **release mode** is unknown.

Two reasons the release-mode answer might differ:

- Linear-scan + per-iter sort/truncate has tight memory access that
  benefits more from SIMD/inlining than heap bookkeeping does. So
  release mode may shift the relative balance.
- The scan-side heap-frontier change (`27bb6af8`) was a clear win;
  same algorithmic shape applied to build was a regression. That's
  asymmetric in a suspicious way — same data structure, same
  invariants, opposite outcome. Worth confirming the asymmetry is
  real and not a debug-mode artifact.

Single A/B test, half a day. If the regression replicates in
release mode, leave reverted as is. If it inverts to a win,
reconsider. Not a blocker; just a "before we declare this option
permanently dead, verify."

### 3. The new prefilter pgrx test is structural, not behavioral

`test_ec_diskann_prefilter_kind_override_switches_prefilter` at
`routine.rs:2038-2118` builds a fixture, calls
`prepare_prefilter` directly with each GUC value, and asserts the
returned `PreparedPrefilter` enum variant matches expectation
(`BinarySidecar` for `Auto`/`BinarySidecar`, `GroupedPq` for
`GroupedPq`).

That's a sound structural check — if the enum variant is wrong, the
scan can't produce correct results — but it doesn't actually
exercise the end-to-end scan path under each GUC value and verify
that results differ. The 29b plan asked for "assert at least one
differing result." The current test asserts the right structural
invariant; it's not wrong, just slightly weaker than the spec.

Acceptable for landing as-is. If a future bug ever makes the two
paths produce identical results for non-trivial inputs (e.g., a
shared closure capturing the wrong prefilter), this test wouldn't
catch it. File as "tighten if a bug ever motivates it."

### 4. pgvectorscale reference comparison was skipped

Task 29c's plan called for installing pgvectorscale DiskANN if
feasible and comparing build/recall/latency on the same corpus.
This was the original Task 29 Phase 1 charter, deferred earlier.
None of the 29c packets attempt this.

The implicit reference is now just `ec_hnsw`, which is fair on its
own terms but doesn't answer the original "how does our DiskANN
compare to other DiskANN impls" question that the user explicitly
asked for as the perf target framing.

If reviewers don't ask for it, fine. If they do ask, the answer is
"HNSW is the in-house relative reference; pgvectorscale comparison
was deferred and remains a follow-up." Worth pre-empting that
question in the merge discussion. Could also slot in as part of the
release-mode re-measurement packet (item 1) — install pgvectorscale,
build the same index, report numbers alongside.

## Risks and follow-ups (none new since round 1)

The risks list from `11099`'s round-1 feedback (vacuum
consistency, GUC end-state, build time, HNSW latency framing, tied
popcount tail) all stand or have been addressed. Updates:

- **Vacuum consistency**: ADDRESSED in 29b.
- **GUC end-state**: ADDRESSED in 29b.
- **Build time**: ADDRESSED in 29c via the release-vs-debug
  finding + active-mask prune. Still slower than HNSW (13.5×) but
  this is the expected trade-off, well-bounded, and observable.
- **HNSW latency framing**: PENDING the release-mode re-measurement.
  If release-mode DiskANN latency drops as expected, this risk
  flips from "DiskANN slower" to "DiskANN at parity or faster" and
  the framing changes. Don't pre-commit either narrative until the
  measurement lands.
- **Tied popcount tail**: STILL UNMEASURED. 0.997 recall at default
  rerank_budget=64 strongly suggests it's not a problem on this
  corpus; file the recipe (secondary tie-break via `tuple.search_code`
  PQ score) as a known mitigation if a future corpus shows recall
  plateauing below 0.99.

## Path to landing

One blocker, one recommended check, two pre-emptive notes:

1. **Blocker**: Release-mode latency re-measurement packet (item 1
   above). Half a day. After this lands and confirms or revises the
   latency numbers, Task 29 is ready for outside review.
2. **Recommended**: Release-mode A/B of the heap-frontier build
   experiment (item 2 above). Half a day. Optional in the sense
   that "stay reverted" is the safe default; doing the check just
   answers an open question that will probably come up.
3. **Pre-empt**: Decide whether to install pgvectorscale for the
   reference comparison (item 4 above). If yes, fold into item 1's
   packet.
4. **Pre-empt**: The structural-vs-behavioral note on the prefilter
   test (item 3 above). Mention in the merge discussion that this
   was a deliberate choice; not a gap in coverage.

After items 1 (and optionally 2) land, the round-3 review is
trivial: confirm the new numbers, sign off on merge. No further
algorithmic changes expected before main.

## Bottom line

The branch is in real merge-ready shape modulo the latency
re-measurement. The work that landed since round 1 is solid:
PreparedPrefilter helper is the right design, vacuum is now
consistent with scan, the active-mask prune is a clean
algorithmic win, and the debug-vs-release catch is the kind of
honest finding that builds confidence rather than undermining it.
The structured timing instrumentation is genuinely useful and
should stay. The remaining gap (debug-mode latency cited for
landing) is a measurement hygiene issue, not a correctness or
design issue. One short re-measurement packet closes it.
