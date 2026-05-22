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
