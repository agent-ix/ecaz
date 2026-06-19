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

## Pending (matrix running on the fixed `.so`)

Per-change A/B verdicts — filled as configs complete, each verified against the
fixed-`.so` backend SHA in its `suite-manifest.json`:

- **111g** rerank reps: f32 vs f16(fixed) vs rabitq4 (table) + index-vs-table
  sidecar latency (codex P1: does index-side full-chain read actually win?).
- **112** lazy rerank: on vs off (expected recall-neutral; latency delta?).
- **113** posting prune: on vs off (row + dense path, codex P2).
- **115** residual: plain vs residual recall-per-probe + matched-recall latency.
- **quant_bits** 1/2/4/8 recall/latency/size.
- **Historical** per-segment: constant-rabitq at baseline/111g/112/113/115.
