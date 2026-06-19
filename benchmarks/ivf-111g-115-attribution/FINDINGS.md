# IVF 111g–115 — Benchmark Findings (in progress)

Empirical findings from the attribution + rerank-matrix bench on the Intel host
(pgrx PG18, real DBpedia openai-3-large dim 1536, 10/50/100k, release `.so`
SHA-verified per run). This file is updated as the matrix completes; the
per-artifact metadata + re-run commands live in `manifest.md`.

## Finding 1 — f16 rerank was broken (recall collapse). FIXED. [proven]

**Discovered by benchmark, missed by static review (mine and codex's).**

The `rerank_format='f16'` path returned recall@10 ≈ 0.6 at every scale while
exact `f32` reached ~0.97–1.0 on the *same* candidate pool — and f16 recall did
**not** climb with nprobe (flat at ~0.62 from nprobe 64→300) while f32 climbed to
0.999. numpy confirmed correct f16 reranking yields recall 1.0 on this data, so
this was an ecaz defect, not f16 precision loss.

**Root cause** (`src/am/ec_ivf/rerank.rs`, `f16_bits_to_f32`): subnormal binary16
values (|x| < 2⁻¹⁴ ≈ 6.1e-5) were decoded with base unbiased exponent **−1
instead of −14**, making them ~2¹⁴× too large (`3.9e-5 → 0.64`). Every embedding
vector carries a few near-zero components; each became ~0.64 garbage that swamped
the f16 inner-product sum and destroyed the ranking. `f32`/`rabitq4` don't use
f16 and were unaffected — exactly matching the per-format bench.

**Why static review missed it:** the 111g recall-parity test only asserted
*index-f16 == table-f16* (both wrong the same way), never *f16 ≈ f32*. The test
was self-consistent but validated the wrong invariant. Only an end-to-end recall
benchmark exposed it.

**Fix** (`fix(ec_ivf): correct subnormal binary16 decode`, commit `0c3efc611` on
`bench-ivf-111g-115-attribution`): subnormal base exponent −14, drop the spurious
+1 in the f32 rebias. 4 lines + 3 regression tests (subnormal round-trip vs
numpy, bit-pattern reference, f16-vs-f32 ranking ≥9/10). All 13 rerank unit tests
green.

**SQL-level proof** — f16 recall@10 at 100k, before → after the fix (table-side
coarse_rerank, rerank_width=64, same index, scan-time scoring):

| nprobe | f16 before | f16 after | f32 (ref) |
|---|---|---|---|
| 64  | 0.6115 | **0.9640** | 0.9655 |
| 128 | 0.6195 | **0.9860** | 0.9875 |
| 200 | 0.6240 | **0.9975** | 0.9990 |
| 300 | 0.6245 | **0.9980** | 0.9995 |

f16 now tracks f32 within ~0.001–0.002. **Proposed action: merge the fix to
`main` (pending operator approval).** Without it, `rerank_format='f16'` /
"halfvec" rerank must not be promoted.

## Finding 2 — "low recall" was a truncated sweep, not the engine. [proven]

Exact f32 recall@10 appeared to cap at 0.965 at 100k — alarming for an exact
rerank. Diagnosis: the standard ec_ivf sweep tops at nprobe=64, which at
nlists=√N (316 for 100k) probes only ~20% of lists; the recall curve is still
*rising* there. Extending the sweep:

| nprobe | recall@10 (100k, f32) | p50 |
|---|---|---|
| 8   | 0.768 | 5.4 ms |
| 32  | 0.923 | 10.1 ms |
| 64  | 0.966 | 16.5 ms |
| 128 | 0.988 | 31.9 ms |
| 200 | **0.999** | 43.4 ms |

`rerank_width` (64/256/512) made **no** difference — the coarse stage already
surfaces the true top-10 within the top-64, so the rerank pool was never the
limit; nprobe was. All attribution configs now sweep `[8,16,32,64,128,200]` so
absolute recall is representative and matched-recall latency comparison is
possible. (This is a measurement-window fix, not a code change.)

## Finding 3 — 111g index-side sidecar is a severe regression (O(N) full-chain read). BUG. [proven]

**codex P1 confirmed and quantified.** `rerank_placement='index'` reads the
**entire** `0x2A` sidecar chain into a `HashMap<heap_tid, payload>` on *every
query* (`scan.rs:2500` → `load_rerank_sidecar_payloads`), then uses only the ~64
survivors. For 100k f16 entries that's ~307 MB read + 100k allocations per query
to serve ~196 KB of survivors — O(N) regardless of `rerank_width`.

**Measured, index-side f16 vs table-side f16 @100k:**

| metric | table-side | index-side | ratio |
|---|---|---|---|
| p50 @ nprobe 8   | 4.7 ms  | **551 ms** | 117× slower |
| p50 @ nprobe 200 | 13.2 ms | **552 ms** | 42× slower |
| p50 vs nprobe    | scales 5→13 ms | **flat ~540 ms** (O(N)) | — |
| index size       | 25.4 MiB | **416 MiB** | 16× bigger |
| recall@10        | ✓ | ✓ (same) | — |

Latency is *flat across nprobe* — the signature of the nprobe-independent
full-chain read. So index-side placement (111g's headline "byte-win", approved on
the `stats_rerank_source_bytes_read` counter) is **40–120× slower and 16× larger**
at identical recall. The counter measured survivor bytes (O(W)); the real work is
O(N). **Verdict: do NOT promote `rerank_placement='index'`; the as-merged
implementation is a regression, not a win.**

**Fix (planned):** survivor-directed bounded read — read only the ~W survivor
payloads, not the whole chain. Requires a sorted directory over the `0x2A` chain
(heap-TID range → block) so blocks without survivors are skipped (build already
writes blocks sorted by heap TID; insert-prepended blocks handled separately).
On-disk format addition (free here — research project, rebuild). This is a real
redesign, sequenced after the benchmark goal completes; then re-bench sidecar-index
for the "after". Table-side stays the correct lean-index default meanwhile.

## Finding 8 — ADR-079 implemented: index-side O(N) bug fixed; rabitq4 now viable, f16 still directory-bound [proven]

Implemented the survivor-directed sidecar directory (ADR-079) — metadata 86→92,
build writes a `(first_tid → block_tid)` directory, scan binary-searches it and
reads only the survivor blocks (full-chain fallback after inserts). Release `.so`
`6078da14`. Re-bench (`artifacts/adr079-sidecar-index/`, fresh build, no inserts):

| index-side @100k | recall np64 / np200 | p50 np8 / np64 / np200 | vs pre-fix (551 ms flat) |
|---|---|---|---|
| **rabitq4** | 0.917 / 0.942 | 7.7 / 9.6 / 16.0 ms | **fixed** — scales, ≈ table-side |
| **f16** | 0.964 / 0.9975 (correct) | 146.8 / 150.2 / 159.2 ms | 3.6× faster, still slow |

- **Correctness proven:** recall matches table-side/f32 at all scales → the
  directed read returns the right payloads. (Index-placement pg_tests + the recall
  match.)
- **rabitq4 index-side fully fixed:** compact payload → many entries/block → small
  directory → bounded read is fast (16 ms, scales with nprobe).
- **f16 still directory-bound:** 3072 B payload → ~2 entries/sidecar-block → ~50k
  blocks → ~74-block directory; the per-block directory read (~140 ms) now
  dominates (flat ~150 ms). Survivor reads are bounded, but f16's directory is
  large. Follow-up lever: denser/2-level directory or a per-relation directory cache.

**Net verdict (unchanged overall):** **table-side f32 remains the best option**
(13 ms, recall 0.999, 25 MiB lean index). ADR-079 removed the catastrophic
regression and makes index-side **rabitq4** a real fast+compact option (but
recall-capped ~0.94, rabitq4 lossy); index-side f16 is recall-correct but slow +
416 MiB. Use table-side f32 by default; index-side rabitq4 where a compact index
matters and ~0.94 recall suffices.

## Per-change verdict table (HEAD, all 8 configs on fixed `.so` 67e1534c, sweep 8–200, @100k)

| Change | Recall effect | Latency / size effect | Verdict |
|---|---|---|---|
| **111g** table f16 (fixed) vs f32 | equal (0.9975 vs 0.9985 @np200) | ~6% slower scoring, same 25.4 MiB index | no benefit vs f32 |
| **111g** table rabitq4 | **lower** (0.942 @np200) | same size, slower | worse — don't use |
| **111g** index-side f16/rq4 | equal recall | **40–120× slower, 16× bigger** (O(N) chain) | **BROKEN → ADR-079** |
| **112** lazy rerank on/off | equal (byte-identical) | neutral (within noise) | **inert** — no win |
| **113** row-path prune on/off | equal (recall-safe ✓) | **+4% faster** @matched recall | keep (only real win) |
| **113** dense-path prune on/off | equal (recall-safe ✓) | ~3% slower (bookkeeping, no pre-prune) | inert → gate to row path |
| **115** residual plain/on | equal (no recall-per-probe gain) | **~9% slower**, same size | net-negative under exact rerank |
| **quant_bits** 1/2/4/8 | all ≈0.999 (1-bit 0.9985) | idx 32.7/52.5/90.4/198.7 MiB; p50 7.8/53.3*/16.6/17.8 ms | **1-bit is the sweet spot** |

\* qb2 p50=53 ms is an anomaly (likely an unoptimized 2-bit scan path) — flagged.

## Overall conclusion (measured)

**Exact `heap_f32` rerank dominates recall@10**, which masks every coarse-side
quant optimization in this lane:
- residual (115) and higher bit-depth (quant_bits) give **no recall gain** —
  exact rerank already fixes the ranking from the coarse top-`rerank_width`.
- **1-bit RaBitQ coarse + exact rerank is the sweet spot**: same recall as 4/8-bit
  at the smallest index (32.7 MiB) and fastest scan (7.8 ms). The actionable win.

On the latency-optimization side:
- **112 lazy is inert** (no skip fires — NoBound + no k-cap + loose bound).
- **113 prune helps only the row path (+4%)**; it is inert on the dense path.
- **111g index-side sidecar is a regression** (O(N) full-chain read), not a win —
  fix is ADR-079 (survivor-directed bounded read).
- **f16 rerank was outright broken** (subnormal decode) → fixed, now ≈ f32.

Net: most of the 111g→115 lane did **not** move the bar at matched recall; the
real, measured takeaways are (a) the f16 fix, (b) 1-bit-coarse + exact-rerank as
the efficient operating point, (c) 113 row-prune ~4%, and (d) the index-side
regression to fix (ADR-079). Exactly the per-change attribution the bench was for.

## Historical per-segment (constant plain-rabitq at each merge commit) [complete]

Each commit rebuilt with its own release `.so` (distinct backend SHAs in
`artifacts/hist-*/suite-manifest.json`) and benched with the identical
constant-rabitq config. Isolates each merge's net effect on the stable rabitq path.

constant-rabitq @100k, p50 latency:

| commit | backend | recall np64/np200 | p50 np64/np200 |
|---|---|---|---|
| baseline 99dc70e53 | 09029337 | 0.9655 / 0.9990 | 16.8 / 45.1 ms |
| 111g 61fd84f95 | c68e5e40 | 0.9655 / 0.9990 | 16.3 / 42.7 ms |
| 112 6d60eec50 | a47be920 | 0.9655 / 0.9990 | 16.7 / 43.6 ms |
| 113 9ddb3be7c | 31085e3c | 0.9655 / 0.9990 | 16.9 / 43.1 ms |

**Conclusion: the lane is neutral on the default rabitq path.** Recall is
byte-identical across all four merges; p50 varies only within run-to-run noise
(~5%, no monotic trend). Each task added opt-in/gated features (coarse_rerank
sidecar, lazy rerank, posting prune toggle, residual) rather than changing the
default RaBitQ scan — so **no merge regressed (or improved) the stable path**.
The real per-feature effects are the HEAD gated A/Bs above; the historical layer
is the safety confirmation (no silent regression slipped in).
