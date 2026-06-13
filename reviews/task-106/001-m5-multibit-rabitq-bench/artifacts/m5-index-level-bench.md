# Task 106 — M5 index-level bench (real DBpedia 10k, PG 18)

Host: Apple M5 (aarch64, NEON). PG 18 (pgrx-managed), ecaz 0.1.1.
Corpus: `ec_hnsw_real_10k` (real DBpedia, 10000 rows, dim 1536) + 200 queries.
Index: ec_ivf, nlists=64. Latency = end-to-end SQL, k=10, 200 iterations.
Counters via `bench latency --task87-candidate-batch-counters`.

This is the index-level companion to the kernel microbench
(`m5-multibit-rabitq-bench.md`): it confirms, in real Postgres at index
scale, that the Task 106 routing engages the right path per bit width and
that the slice-3 Auto-gate fix works.

## IVF × RaBitQ bit sweep (the multi-bit routing)

| bits | p50 (nprobe=16) | block-kernel counters | path taken |
| ---- | --------------- | --------------------- | ---------- |
| 1    | 32.6 ms         | isa=neon, kernel_candidates=530088, scalar=0, width_ge32=2148 | rabitq32 block kernel |
| 2    | 46.0 ms         | isa=neon, kernel_candidates=530088, scalar=0, width_ge32=2148 | **multi-bit block kernel (Task 106)** |
| 4    | 46.2 ms         | none                  | arithmetic estimator (NeonBits4) |
| 8    | 34.5 ms         | none                  | arithmetic estimator (NeonBits8) |

**Routing confirmed end-to-end.** bits=1 and bits=2 drive the rabitq32
block kernel (counters present, NEON, predominantly 32-wide flushes);
bits=4 and bits=8 emit no block-kernel counters — they take the
per-candidate arithmetic estimator, exactly as the M5 microbench evidence
dictated (bits=4 NeonBits4 beats the block kernel, so it is not block-routed).

bits=2 kernel scoring time (6902 ms / 200 queries) > bits=1 (4306 ms) — the
2-bit codes are longer — but it *engages* the block kernel rather than
falling to true scalar, which is the win the microbench measured (~3×).

e2e p50 is similar for bits=2/4 (~46 ms) and lower for bits=1/8 (~33 ms):
the scoring kernel is a fraction of the rerank-dominated scan, so per-bit
e2e tracks code length / rerank more than the scoring path — consistent
with ADR-077's scoring-share-saturation observation.

## IVF × TurboQuant Auto-gate (slice 3)

Default index (no `storage_format` reloption → `Auto`, resolves to
TurboQuant), nprobe=16:

| `ec_ivf.scratch_soa_batch_decode` | p50 | batch counters |
| --------------------------------- | ---- | -------------- |
| off | 23.5 ms | `surface=ivf flushes=0 candidates=0` (per-candidate pruning path) |
| on  | 70.0 ms | `quant=turboquant isa=neon kernel_candidates=530085 width_ge32=2148` (batch engages) |

**Slice-3 fix confirmed:** with the gate fixed, an `Auto`-built index now
engages the batch path when the GUC is on. Before the fix the gate rejected
`Auto`, so the 10k profile (default `Auto`) emitted zero batch counters
regardless of the GUC — the exact ADR-077 §9.3 symptom. The fix makes `Auto`
behave like explicit TurboQuant.

**Honest nuance the bench surfaced:** batch-on is ~3× *slower* e2e here
(70 vs 23.5 ms) because the batch path forfeits the per-candidate suffix-max
pruning (ADR-077 §7.1's documented trade). The GUC default stays off, so
default indexes keep the pruning fast path; the gate fix only makes the
engagement consistent when the GUC is enabled. Whether batch-on should be
the IVF default (ADR-077 §4) is a separate enablement decision this fix does
not change.

## Not yet run (index level)

SPIRE × RaBitQ GUC A/B (`ec_spire.candidate_batch_scoring` on/off) — the
slice-1 surface. The IVF surfaces (the bulk of the Task 106 changes) are
index-level validated above; the SPIRE assignment-scoring migration is
covered by its passing unit test and a follow-up index-level A/B.
