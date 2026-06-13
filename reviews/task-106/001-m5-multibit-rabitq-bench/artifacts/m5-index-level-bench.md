# Task 106 — M5 index-level bench (real DBpedia 10k, PG 18, RELEASE backend)

Host: Apple M5 (aarch64, NEON). PG 18 (pgrx-managed, port 28818), ecaz 0.1.1,
**release-installed extension** (`cargo pgrx install --release`). Corpus:
`ec_hnsw_real_10k` (real DBpedia, 10000 rows, dim 1536) + 200 queries. Index:
ec_ivf, nlists=64. Latency = end-to-end SQL, k=10, 200 iterations.

> **Backend note.** All numbers below are from the **release** backend. An
> earlier ad-hoc pass used a debug-installed extension, which inflated
> latencies ~20–30× and, critically, *inverted* the IVF Auto-gate conclusion
> (see below). The `ecaz bench suite` runner rejects a debug backend, which
> caught this. The authoritative, reproducible source for the IVF RaBitQ bit
> sweep is `suite/results.jsonl` + `suite/latency-ivf-rb{1,2,4,8}.log`
> (config: `crates/ecaz-cli/suites/task106-m5-ivf-rabitq-multibit.json`).
> The Auto-gate and SPIRE A/B used the direct `ecaz bench latency` surface;
> raw capture in `raw-index-autogate-spire-ab-release.log`.

## IVF × RaBitQ bit sweep (the multi-bit routing) — `suite/`

| bits | p50 (nprobe=16) | block-kernel counters | path taken |
| ---- | --------------- | --------------------- | ---------- |
| 1    | 0.67 ms | `isa=neon kernel_candidates=530088 scalar=0 width_ge32=2148` | rabitq32 block kernel |
| 2    | 2.09 ms | `isa=neon kernel_candidates=530088 scalar=0 width_ge32=2148` | **multi-bit block kernel (Task 106)** |
| 4    | 1.15 ms | none | arithmetic estimator (NeonBits4) |
| 8    | 0.96 ms | none | arithmetic estimator (NeonBits8) |

**Routing confirmed end-to-end on release.** bits=1 and bits=2 drive the
rabitq32 block kernel (counters present, NEON, predominantly 32-wide flushes);
bits=4 and bits=8 emit no block-kernel counters — they take the per-candidate
arithmetic estimator, exactly as the microbench evidence dictated (bits=4
NeonBits4 beats the block kernel, so it is not block-routed).

bits=2 e2e p50 (2.09 ms) is the highest — its 2-bit codes are longer and the
block kernel scores 530k candidates (kernel time 302 ms / 200 q) — but it
*engages* the block kernel rather than falling to true scalar. bits=1 is
fastest (shortest codes). e2e tracks code length / rerank as much as the
scoring path.

## IVF × TurboQuant Auto-gate (slice 3)

Default index (no `storage_format` → `Auto`, resolves to TurboQuant), nprobe=16:

| `ec_ivf.scratch_soa_batch_decode` | p50 | batch counters |
| --------------------------------- | ---- | -------------- |
| off | 2.77 ms | `surface=ivf flushes=0 candidates=0` (per-candidate path) |
| on  | 1.16 ms | `quant=turboquant isa=neon kernel_candidates=530085 width_ge32=2148` (batch engages) |

**Slice-3 fix confirmed:** with the gate fixed, an `Auto`-built default index
engages the batch path when the GUC is on. Before the fix the gate rejected
`Auto`, so the 10k profile emitted zero batch counters regardless of the GUC —
the exact ADR-077 §9.3 symptom. The fix makes `Auto` behave like explicit
TurboQuant.

**On release, batch-on is 2.4× FASTER** (1.16 vs 2.77 ms). (An earlier
debug-backend pass wrongly showed batch-on ~3× *slower* — the debug kernel's
overhead masked the win. The release result agrees with ADR-077 §4's
"batch-on wins IVF TurboQuant" enablement decision.)

## SPIRE × RaBitQ GUC A/B (slice 1)

SPIRE rabitq index (default bits=4), nprobe=16:

| `ec_spire.candidate_batch_scoring` | p50 | counters |
| ---------------------------------- | ---- | -------- |
| off | 8.57 ms | `surface=spire flushes=0 candidates=0` |
| on  | 8.41 ms | `surface=spire flushes=0 candidates=0` |

**Honest finding:** ~0% e2e delta (within noise) and zero candidate-batch
counters on this lane. The slice-1 migration makes the SPIRE rabitq path
correct and counter-capable (bits=1 → block kernel; verified by unit test),
but SPIRE is bits=4 by default (estimator path, no block-kernel counters) and
its e2e is routing/rerank-dominated, so assignment scoring is negligible. The
ADR-077 §9.1 "inert toggle (~0% on/off)" is largely inherent to SPIRE at this
workload — a code-correctness/attribution change, not a measurable e2e win.

## Remaining (other hosts)

AVX2 (Intel) — may revisit bits=4 block routing via the hardware gather;
SVE/NEON-cap (G4); the HNSW grouped-PQ flush-width histogram; recall@k vs
ground truth and scoring-share at production scale.
