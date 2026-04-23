# Review Request: Task 25 Slice 8 — ADR-045 Stage 1 Gate Verdict (FAIL, DBpedia-10k)

Scope: documentation only. Records the ADR-045 Stage 1 recall gate
run on the canonical DBpedia-OpenAI3-1536d real corpus (10k slice),
produced by running the slice-7 `ecaz quant feasibility` harness.

Task: `plan/tasks/25-rabitq-quantizer.md`, Phase 2 decision gate.

Branch: `task25-rabitq-stage1-phase0` (slice 8 builds on `327e003`).

Artifacts in this packet:
- `run-dbpedia-10k.txt` — verbatim harness output.

## Verdict

**FAIL** against the ADR-045 Stage 1 gate (recall@10 within 1 pp of
exact, at PQ4-parity storage). Recall gap is **10.65 pp** — outside
MARGINAL (1–2 pp) and firmly in the FAIL band (>2 pp).

## Reproducer

Canonical TSVs pre-staged under
`target/real-corpus/ec_hnsw_real_10k/` (sourced from
`Qdrant dbpedia-entities-openai3-text-embedding-3-large-1536-1M` via
`ecaz corpus prepare`).

```
./target/release/ecaz quant feasibility \
    --quantizer rabitq \
    --corpus-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_corpus.tsv \
    --queries-file target/real-corpus/ec_hnsw_real_10k/ec_hnsw_real_10k_queries.tsv \
    --dim 1536 --top-k 10
```

Wall time ~19 s on this box. Output saved as `run-dbpedia-10k.txt`.

## Numbers

```
loaded: 10000 corpus vectors, 200 queries (dim=1536, top_k=10, seed=42)
storage: RaBitQ code 200 B, PQ4 code 768 B (parity ratio 3.84x)

recall@10 mean: 0.8935
bound  mean=0.612  p50=0.612  p99=0.630
error  mean=0.268  p50=0.278  p99=0.352
tightness (error / bound) mean: 0.437
```

Compared to the slice-7 synthetic iid-Gaussian run at the same
`(10k, 200, D=1536)` shape:

| metric                 | synthetic iid | DBpedia-10k |
|------------------------|---------------|-------------|
| recall@10              | 0.2535        | 0.8935      |
| mean bound             | 0.602         | 0.612       |
| mean error             | 0.012         | 0.268       |
| tightness (err/bound)  | 0.020         | 0.437       |

The real corpus swings recall from "no signal" (adversarial iid
Gaussian) to "real but short of the gate" (DBpedia cluster structure
makes top-K recoverable, but not at 99+% yet). Tightness jumps 22×
because DBpedia embeddings concentrate most of their energy in a
subspace the sign-bit approximation captures coarsely; the residual
norm `√(||c||² − α²·D)` stays close to the Cauchy-Schwarz worst
case rather than averaging down.

## Decision per the task rubric

From `plan/tasks/25-rabitq-quantizer.md`:

> **Pass** (≤1 pp) → publish study, unblock Symphony Stage 2 (task 26
> successor).
> **Marginal** (1–2 pp) → keep the module; return to OPQ (task 20) to
> close the gap via a learned rotation.
> **Fail** (>2 pp) → shelve Stages 2–3 of ADR-045, keep the module
> as the ADR-031 prefilter successor, record the null result.

10.65 pp is FAIL territory. **The clean action is to shelve task 27
Stages 2 & 3 at the default configuration** and keep `src/quant/rabitq.rs`
as the ADR-031 prefilter successor — which is exactly the fallback
role ADR-031 retains in the task doc's "Supersedes the ADR-031
prefilter design in scope; ADR-031 remains as a fallback posture if
the gate fails" clause.

## Why this is a real FAIL, and what it does NOT say

Three honest caveats the reviewer should weigh before applying the
shelve decision:

1. **Rotation.** `SrhtRotation` reuses the `ProdQuantizer`'s SRHT
   signs, which were trained to decorrelate coordinates for PQ
   compression. That is **not** a RaBitQ-optimal rotation. A
   learned rotation (task 20 OPQ) or even a dedicated random
   rotation with higher Johnson-Lindenstrauss quality could shift
   the bound distribution enough to change the verdict. The task
   doc's MARGINAL path explicitly calls this out. **This FAIL says
   "at this rotation, this estimator"**, not "RaBitQ cannot hit
   the gate."

2. **Estimator form.** The slice-4 estimator uses
   `α_c = mean(|c_i|)` and a symmetric Cauchy-Schwarz bound. The
   RaBitQ paper's canonical formulation stores a per-vector
   `⟨c_u, sign(c_u)⟩ / √D` cosine scalar and uses the query's
   rotated-norm-times-that-cosine product; it trades a slightly
   larger per-vector tail (still 8 B at D=1536) for a tighter
   bound. If a reviewer wants to verify the FAIL is not an
   estimator bug, a faithful port is a 1–2 day follow-up.

3. **Corpus size.** The gate spec names "50k and 1M real seams."
   This run is 10k. Smaller corpora sometimes hide recall headroom
   (fewer near-ties at the K-th boundary). Running 50k / 1M would
   either confirm the FAIL at larger N or reveal a recall cliff
   that the 10k slice flattens. `ecaz corpus prepare` can emit the
   other slices from the same parquet; the harness is ready.

My read: caveat 1 is the biggest. OPQ (task 20) should run before
the shelve is final. Caveats 2 and 3 are worth a single combined
follow-up slice but are unlikely to flip the verdict on their own
at the observed 10.65 pp gap.

## Recommended next steps

In order of cost vs. information gained:

1. **Run the 50k and 1M slices with the same harness.** Cost: two
   harness invocations; numbers end up next to this one in a
   follow-up packet. Clarifies whether 10k hides the real answer.
2. **Run the DBpedia harness against a RaBitQ-paper-canonical
   estimator.** Cost: 1–2 day port in `src/quant/rabitq.rs`
   (behind a `--estimator canonical` flag in the harness so the
   two forms live side-by-side).
3. **Run against OPQ-rotated corpus (task 20).** Gated on task 20
   Phase 1 landing. If the verdict flips with OPQ, the roadmap
   consequence is "Stage 2 depends on task 20" — not a shelve.
4. **Only after those**: if recall is still >2 pp short, formally
   record the ADR-045 shelve in a status update and freeze task 27
   at "not started".

## Consequences for task 27 (Symphony Stages 2 & 3)

Per the slice-6 handoff contract, task 27 was gated on both (a)
contract sign-off and (b) a PASS verdict. Condition (b) is not met
at this run. Task 27 kickoff is **deferred** pending the follow-up
investigations above. The contract itself stays valid — the API
surface `src/quant/rabitq.rs` exposes does not change based on the
verdict; only task 27's start date does.

## What slice 8 itself demonstrates

Outside the verdict: the `ecaz quant feasibility` harness
(slice 7) produced a gate-decision-grade result on real data on the
first real-corpus invocation, end-to-end, in 19 s. The harness is
battle-tested. Future tasks (20, 22, 23, 27) will reuse it without
rebuilding the loader, the brute-force truth loop, or the summary
formatter.
