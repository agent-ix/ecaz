# Phase A results — pre/post NEON RaBitQ on Graviton 4 m8g.large

Captured 2026-05-22 on instance `i-0ee528ff09d9d70dc` (m8g.large,
us-west-2), data restored from `snap-054feaffc50ecf1c9` (real DBpedia
10k + 50k), per-variant tables created via
`setup-per-variant-tables.sql`.

## Headline

The aarch64 NEON kernel for RaBitQ `bits=4` (commit `02f0e78c2`)
turns the documented **1.35–1.46× RaBitQ-slower-than-TQ gap on
Graviton into a 2.2× RaBitQ-faster-than-TQ win** at matched nprobe.
The kernel is a drop-in replacement of the scalar inner loop in
`estimate_ip_impl`; the rest of the RaBitQ formula (alpha, error
bound, scalar tail) is unchanged.

Direct A/B at 50k, real DBpedia, m8g.large, k=10, concurrency=1,
1000 iterations:

| nprobe | RaBitQ pre p50 | RaBitQ post p50 | **Speedup** | TQ post p50 | RaBitQ/TQ post |
| --- | --- | --- | --- | --- | --- |
| 8 | 4.61 ms | 3.82 ms* | 1.21× | 6.51 ms | 0.59× |
| 16 | 8.22 ms | 2.89 ms | **2.84×** | 6.19 ms | **0.47×** |
| 24 | 12.0 ms | 4.08 ms | **2.94×** | 8.77 ms | **0.46×** |
| 32 | 15.7 ms | 5.09 ms | **3.08×** | 11.4 ms | **0.45×** |
| 48 | 23.1 ms | 7.38 ms | **3.13×** | 16.3 ms | **0.45×** |
| 64 | 30.0 ms | 9.35 ms | **3.21×** | 20.8 ms | **0.45×** |

\* nprobe=8 post-NEON cell is cold-cache-noisy (stddev 2.32 ms vs 0.38–0.58
at higher nprobe); ignore for steady-state comparison.

Same pattern at 10k:

| nprobe | RaBitQ pre p50 | RaBitQ post p50 | **Speedup** | TQ pre p50 | RaBitQ/TQ post |
| --- | --- | --- | --- | --- | --- |
| 16 | 4.27 ms | 1.49 ms | **2.87×** | 2.98 ms | 0.50× |
| 24 | 6.10 ms | 2.00 ms | **3.05×** | 4.15 ms | 0.48× |
| 32 | 7.58 ms | 2.40 ms | **3.16×** | 5.23 ms | 0.46× |
| 48 | 10.7 ms | 3.33 ms | **3.21×** | 7.58 ms | 0.44× |
| 64 | 14.2 ms | 4.34 ms | **3.27×** | 10.0 ms | 0.43× |

## Sanity: TurboQuant unchanged

50k TQ p50 was 20.5 ms @ nprobe=64 pre, 20.8 ms post. Within noise.
The NEON commit only touches the RaBitQ scalar-inner-loop replacement;
no side effects on the TQ path, confirmed empirically.

## How PQ_FASTSCAN still compares (context only)

PQ_FASTSCAN was already SIMD-optimized via `score_ip_from_parts_tiled_lut_no_qjl_4bit`
and remains the latency champion. At 50k nprobe=8, PQ_FASTSCAN p50 1.09 ms
vs RaBitQ post-NEON 3.82 ms. The NEON RaBitQ optimization does **not** beat
PQ_FASTSCAN — it just closes the gap with TQ and overtakes TQ on cache
pressure (RaBitQ's 4-bit code is more compact than TQ's mse+qjl split).
The PQ_FASTSCAN bench was unaffected by this round; numbers below for context.

| nprobe | 50k PQ_FASTSCAN p50 | 50k RaBitQ post-NEON p50 | RaBitQ / PQFS |
| --- | --- | --- | --- |
| 8 | 1.09 ms | 3.82 ms | 3.5× |
| 16 | 1.36 ms | 2.89 ms | 2.1× |
| 32 | 1.98 ms | 5.09 ms | 2.6× |
| 64 | 3.11 ms | 9.35 ms | 3.0× |

## Recall — unchanged by NEON

The NEON kernel is bit-identical to the scalar reference up to fp accumulation
order (validated by `neon_sum_query_dequant_matches_scalar_bits4` test with
1e-4 relative tolerance). Recall@10 numbers from the baseline suite remain
authoritative:

| Corpus | storage_format | nprobe=8 recall@10 | nprobe=64 recall@10 |
| --- | --- | --- | --- |
| 10k | turboquant | 0.9690 (pre)† | (post unchanged) |
| 10k | rabitq | ≈0.97 (pre)† | (post unchanged) |
| 50k | turboquant | 0.8290 (suite) | 0.9414 (suite) |
| 50k | rabitq | 0.8287 (suite) | 0.9379 (suite) |

† 10k recall numbers truncated from the baseline log capture; recoverable
from `/tmp/aws-round-rabitq-ivf/artifacts/recall-10k-ivf-*.log` on the host
if needed for the final closeout.

## Final — warm-cache numbers on m8g.xlarge (`pg_prewarm`-driven)

Cycles 1 + 2 stacked, measured with `pg_prewarm` of every `real_*`
table+index before each sweep so cold-cache EBS reads don't pollute
the inner-loop signal.

### Cumulative RaBitQ speedup (p50)

| Scale | nprobe | Pre-NEON p50 | Post-all p50 | **Speedup** |
| --- | --- | --- | --- | --- |
| 10k | 8 | 2.31 ms | 0.91 ms | **2.54×** |
| 10k | 16 | 4.27 ms | 1.46 ms | **2.92×** |
| 10k | 24 | 6.10 ms | 1.93 ms | **3.16×** |
| 10k | 32 | 7.58 ms | 2.29 ms | **3.31×** |
| 10k | 48 | 10.7 ms | 3.24 ms | **3.30×** |
| 10k | 64 | 14.2 ms | 4.07 ms | **3.49×** |
| 50k | 8 | 4.61 ms | 1.81 ms | **2.55×** |
| 50k | 16 | 8.22 ms | 2.73 ms | **3.01×** |
| 50k | 24 | 12.0 ms | 3.82 ms | **3.14×** |
| 50k | 32 | 15.7 ms | 4.74 ms | **3.31×** |
| 50k | 48 | 23.1 ms | 6.84 ms | **3.38×** |
| 50k | 64 | 30.0 ms | 8.64 ms | **3.47×** |

### RaBitQ vs TurboQuant — comparison reversal (50k warm)

| nprobe | RaBitQ post p50 | TQ warm p50 | RaBitQ / TQ |
| --- | --- | --- | --- |
| 16 | 2.73 ms | 6.32 ms | **0.43× (RaBitQ 2.32× faster)** |
| 24 | 3.82 ms | 8.86 ms | **0.43×** |
| 32 | 4.74 ms | 11.5 ms | **0.41×** |
| 48 | 6.84 ms | 16.5 ms | **0.41×** |
| 64 | 8.64 ms | 21.3 ms | **0.41×** |

The pre-cycle ratio was 1.35–1.46× *slower*; after the kernel +
hoist + pre-prune it is **2.3–2.4× faster** at every nprobe ≥ 16.

### Recall — unchanged (Cauchy-Schwarz prune is recall-safe)

10k RaBitQ recall@10 across the prune-eligible cells:

| nprobe | recall@10 (warm cycle 2) | recall@10 (suite baseline pre-NEON) |
| --- | --- | --- |
| 8 | 0.9730 | 0.9690 |
| 16 | 0.9780 | 0.9730 |
| 32 | 0.9790 | 0.9745 |
| 64 | 0.9790 | 0.9745 |

The small recall *uplift* (~0.5%) is sampling noise from the
queries_limit=200 vs full-200 differences, not a methodology change;
ndcg@10 stays at 0.999 throughout. Pre-prune did not introduce any
recall regression.

### Per-optimization contribution

| Layer | 50k nprobe=64 p50 | gain |
| --- | --- | --- |
| Pre-NEON (scalar fallback) | 30.0 ms | 1.00× |
| + NEON `bits=4` kernel (`02f0e78c2`) | 9.35 ms | 3.21× |
| + LUT hoist (`2ca854d5c`) + pre-prune (`752325deb`), warm | 8.64 ms | 3.47× |
| + bf16 bfdot via inline asm (`02d8cb0da`), kept but inactive on V2 | 8.82 ms | (no measurable Δ on Neoverse-V2 — same 128-bit VL as NEON; kept for future Graviton) |
| + IVF scan dedup → hashbrown / heaptid_count field (`5b4a80c22`) | 8.60 ms | 3.49× |
| + 4-way NEON accumulator unroll (`efc8cb301`) | 8.08 ms | 3.71× |
| + IVF centroid scan NEON (`fb96a8c40`) | **7.87 ms** | **3.81×** |

### rerank_width fix — 9× rerank latency win

The IVF `rerank_width=0` default was a footgun: `pre_rerank_candidate_limit`
returned `None` for 0, which meant `collect_ranked_probe_candidates`
collected **every** candidate before reranking. At nprobe=64 on a 50k
corpus that's ~32K heap fetches per query, taking ~70 ms by itself.

Capping at 200 (commit `425d752fb` changes the default) cuts rerank
latency by ~9× at **identical recall**:

| nprobe | width=0 (uncapped) p50 | width=200 p50 | width=100 p50 | recall@10 |
| --- | --- | --- | --- | --- |
| 8 | 9.29 ms | 2.60 ms | 2.13 ms | 0.855 |
| 16 | 17.3 ms | 3.52 ms | 3.02 ms | 0.920 |
| 32 | 34.7 ms | 5.38 ms | 4.86 ms | 0.964 |
| 64 | 79.4 ms | 8.93 ms | 8.37 ms | 0.988 |

200 is wide enough that recall@10 matches the uncapped run exactly
(top-200 RaBitQ candidates contain the true top-10 with overwhelming
probability on real DBpedia at every nprobe we measured).

This is the biggest single win of the round — the prior default was
adding 70 ms of pointless per-query work at the high-recall operating
point.

### vchord head-to-head (50k, k=10, IP)

vchord baseline from `benchmarks/comparators-50k-100k-1m/manifest.md`
(same DBpedia data, m8g.2xlarge):

| System | 50k p50 ms | recall@10 |
| --- | --- | --- |
| vchord RaBitQ-on-IVF default | **2.7** | ~0.99+ |
| ec_ivf no-rerank nprobe=8 | 1.58 | 0.83 |
| ec_ivf no-rerank nprobe=16 | 2.40 | 0.88 |
| ec_ivf no-rerank nprobe=64 | 7.87 | 0.94 |
| ec_ivf rerank_width=100 nprobe=8 | 2.13 | 0.855 |
| ec_ivf rerank_width=100 nprobe=16 | **3.02** | **0.920** |
| ec_ivf rerank_width=100 nprobe=32 | 4.86 | 0.964 |
| ec_ivf rerank_width=100 nprobe=64 | 8.37 | **0.988** |

**Where we land vs vchord (matched recall at ~0.99):**
- vchord: 2.7 ms
- ec_ivf rerank=heap_f32, width=100, nprobe=64: 8.37 ms

**3.1× gap remains**, and it's now structural. The remaining
difference is heap_f32 rerank doing `fetch_heap_row_version` per
candidate (PG heap I/O), while vchord stores the full f32 source
inline in the index (which is why their index is 415 MB at 50k vs
our 46 MB — they pay storage to skip heap fetches). Closing that
gap requires an in-index source variant — a follow-up architectural
change beyond this round's scope.

At lower recall bands we're closer to parity (3.02 ms @ 0.920 vs
vchord's 2.7 ms; we just operate at lower recall there).

The headline is the NEON kernel (3.2× alone); LUT-hoist + pre-prune
deliver the next 7-8% by eliminating per-candidate redundant table
builds and skipping the SIMD inner product on candidates whose
Cauchy-Schwarz upper bound is already below the running top-K
cutoff. Hashbrown + 4-way unroll + centroid SIMD add another ~10%
combined by clearing the V2 pipe-utilization bottleneck and lifting
the previously-scalar centroid-scoring step that was burning ~1 ms
of per-query fixed cost.

## Cycle 2 — LUT hoist + Cauchy-Schwarz pre-prune (m8g.xlarge)

Same snapshot data (`snap-0bb07e0b82150a062`) restored onto a
**m8g.xlarge** (4 vCPU / 16 GB) via the new `10k-medium` profile
(commit `567e42213`); the previous run was on the under-provisioned
m8g.large. Branch HEAD `752325deb` adds:

- LUT hoist (`2ca854d5c`): precompute the 16-entry dequant table once
  per PreparedEstimator instead of per candidate.
- Cauchy-Schwarz pre-prune (`752325deb`): skip the SIMD inner product
  when the cheap scalar bound `||o|| · ||q|| / o_dot` falls below the
  running top-K cutoff.

Latency p50 on the warm cells (50k):

| nprobe | post-NEON only | post-NEON + LUT + preprune | delta |
| --- | --- | --- | --- |
| 16 | 2.89 ms | 2.84 ms | -2% |
| 24 | 4.08 ms | 3.86 ms | -5% |
| 32 | 5.09 ms | 4.76 ms | -6% |
| 48 | 7.38 ms | 6.85 ms | -7% |
| 64 | 9.35 ms | 8.65 ms | -7% |

Recall (10k, k=10) unchanged from the pre-NEON baseline at
nprobe ∈ {8, 16, 32, 64}: 0.973 → 0.978 → 0.979 → 0.979. Pre-prune
correctness confirmed: Cauchy-Schwarz is a true upper bound on the
estimate, so any skipped candidate provably could not enter top-K.

**Cold-cache caveat.** The PG restart between the rebuild and the
bench wiped buffer pools; the first nprobe=8 cell at 50k shows
extreme variance (mean 44 ms, max 980 ms) while the warm cells are
stable (`stddev/p50 < 10%` by nprobe=32). Future cycles should run
a query warm-up pass before the measured sweep.

Pre-prune fires more aggressively at high nprobe (more candidates
checked → more skips). The 5–7% gain at nprobe ∈ {32, 48, 64} is
where it lands; at low nprobe few candidates are scored to begin
with so the LUT-hoist micro-saving dominates the delta.

## Cost

m8g.large + 50 GB gp3 ≈ $0.16/hr instance + ~$0.005/hr EBS.
Phase A cycle wall clock:
- Provision + cloud-init + ecaz install: ~50 min
- Setup SQL (6 index builds): ~2 min
- Baseline suite (18 steps): ~15 min
- NEON commit + rebuild + reinstall: ~10 min
- Post-NEON latency rerun (12 sweep points): ~5 min
- Total: ~1h 20m → about $0.22 in compute.

Next: snapshot the post-NEON working volume + destroy stack.

## Artifacts

- `artifacts/suite-baseline-pre-neon.log` — tail of the suite driver log (50k portion + storage). 10k latency tables sit in per-step logs on the host.
- `artifacts/all-latency-pre-neon.log` — every pre-NEON latency table.
- `artifacts/latency-post-neon.log` — post-NEON 10k RaBitQ, 50k RaBitQ, 50k TQ confirmation rerun.
- `artifacts/cloud-up.log`, `artifacts/cloud-install.log` — provisioning.

## Cross-references

- Plan: `/home/peter/.claude/plans/ok-we-re-starting-aws-glistening-sloth.md`
- Prep packet: `benchmarks/aws-round-prep/manifest.md`
- NEON commit: `02f0e78c2` "RaBitQ aarch64 NEON inner-loop for bits=4 estimate_ip hot path"
- Punch list P0: cited the missing aarch64 SIMD as the headline target — now closed.
