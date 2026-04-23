# Review Request: Task 25 Slice 14 — Symphony Prerequisite Finding + Task 27 Un-Defer

Scope: documentation only. Captures two findings from reading the
SymphonyQG paper (SIGMOD 2025, arXiv:2411.12229) that change both
the task-25 closing posture and the task-27 start date:

1. **SymphonyQG uses 1-bit RaBitQ**, not Extended RaBitQ. The
   slice-12 q-bit work is correct but off Symphony's critical path.
2. **Symphony's 0.99 recall comes from per-vertex centered
   quantization + multi-visit beam search** — not from a different
   quantizer. Our slice-10 "FAIL at 10 pp" verdict was measuring the
   wrong configuration for Symphony.

Artifacts:
- `~/dev_bak/papers/symphonyqg-2025-sigmod-arxiv-2411.12229.pdf`
- `~/dev_bak/papers/rabitq-2024-sigmod-arxiv-2405.12497.pdf`
- `~/dev_bak/papers/extended-rabitq-2025-sigmod-arxiv-2409.09913.pdf`

Branch: `task25-rabitq-stage1-phase0` (slice 14 builds on `3431476`).

## Finding 1 — Symphony is 1-bit RaBitQ

Symphony paper §2.2 equation (3):

```
C_rand := { Px | x[i] ∈ {+1/√D, −1/√D}, i = 1, 2, ..., D }
```

and "the quantization code can be represented as a D-bit string,
with 1 bit for each dimension." Symphony cites [35] = **original
RaBitQ** (SIGMOD 2024, arXiv:2405.12497), not Extended RaBitQ.

**Consequence for task 25:** the q-bit work (slice 12), Lloyd-Max
scalar quantizer, and q-aware ε-bound work are all **off
Symphony's critical path**. They remain valuable for non-Symphony
consumers (DiskANN in-memory tier, ADR-031 prefilter successor,
offline eval) and are parked as ADR-045 open follow-ups.

**Consequence for the slice-6 handoff contract:** the `bits_per_dim`
story in the contract is accurate but Symphony will consume at
`bits = 1` exclusively. No contract amendment needed; task 27
reads the contract as-is.

## Finding 2 — Symphony quantizes per-vertex residuals, not absolutes

Symphony paper §3.1.1:

> "A natural idea is to normalize the vectors of the neighbors
> with the vector of the vertex (i.e., using the vector of the
> vertex as the center vector c). Then with the normalized data
> vectors, we can compute the quantization codes of RaBitQ and
> store them on the side of the vertex."

Per-vertex encoding: for each vertex `v` and each neighbor `n`,
encode `x_b := sign((n − v) / ||n − v||)`. Same neighbor `n`
appears under multiple vertices with different codes.

The decomposition that makes this affordable at query time
(equation 6 in the paper):

```
⟨x̄, P⁻¹q⟩ = (1/||q_r − c||) · (⟨x̄, P⁻¹q_r⟩ − ⟨x̄, P⁻¹c⟩)
```

- `⟨x̄, P⁻¹c⟩` is independent of the query → pre-compute at
  index-build time, store one f32 per vertex.
- `⟨x̄, P⁻¹q_r⟩` is independent of the center → prepare one LUT
  per query, share across all visited vertices.
- `||q_r − c||` is the only per-vertex-visit arithmetic; trivial.

**Symphony §3.1.2 (implicit re-ranking):**
> "we always append the neighbor along with its estimated
> distance into the beam set unless it has been visited. As a
> result, the same vertex may be added to the beam set multiple
> times (along with different estimated distances)."

Multi-visit beam search means the true NN gets multiple
independent estimates during search; its min-estimate is
extremely likely to be smallest in the beam. No explicit exact
rerank — the beam dynamics plus the error bound do the work.

**Consequence for the slice-10 verdict:** our 0.8975 recall on
absolute-encoded DBpedia-10k was the wrong gate. Residual
encoding + multi-visit beam is how Symphony hits 0.99. The real
Symphony gate is an end-to-end system test (task-27 territory),
not a standalone quantizer test.

## Task 27 un-defer

Slice 10 recommended shelving task 27 based on the 1-bit
absolute-encoding FAIL. That recommendation is **withdrawn**. Our
`src/quant/rabitq.rs` module at slice 9 is what Symphony needs as
the quantizer primitive; task 27's job is to implement the
centering seam + multi-visit beam search on top.

Task 27 kickoff is now **unblocked from the task-25 side**. The
remaining gate on task 27 is the slice-6 handoff contract
review.

## API surface task 27 will add to `rabitq.rs`

Proposed, to be refined during task-27 implementation. Lands as
inherent methods on `RaBitQQuantizer`, not on the `Quantizer` /
`QueryScorer` traits (traits stay unary for non-Symphony
consumers).

```rust
// Build-time, per-vertex.
pub fn encode_code_centered(&self, v: &[f32], center: &[f32]) -> Box<[u8]>;
pub fn prepare_center_scalars(&self, center: &[f32]) -> CenterScalars;

// Query-time, center-independent.
pub fn prepare_scorer_centered(&self, q: &[f32]) -> CenteredScorer;

// Per-vertex-visit.
impl CenteredScorer {
    pub fn score_at(
        &self,
        code: &[u8],
        center_scalars: &CenterScalars,
        center: &[f32],
    ) -> DistanceEstimate;
}
```

Existing c=0 trait path unchanged. Non-Symphony consumers use the
trait surface exactly as today.

## ADR-045 updates in this slice

- Added "Higher-bit quantization (Extended RaBitQ)" subsection to
  the "Open follow-ups" section with the three specific gaps
  between our slice-12 q-bit code and the Extended RaBitQ paper
  (scalar quantizer, bound, bit-level scoring). Parked behind a
  clear trigger condition.
- Added "Per-center RaBitQ API (Symphony Stage 2 prerequisite)"
  subsection to the same section with the API shape, the
  equation-6 decomposition, and the rationale for keeping the
  centered path off the `Quantizer` trait.
- Papers archived in `~/dev_bak/papers/` (not committed to repo
  since outside the project tree and large binary).

## What this slice does NOT do

- No code changes. The API surface sketched above lands when
  task 27 starts — this packet captures the design so task 27
  doesn't rediscover it.
- No contract amendment to `review/20005-task25-task27-handoff-contract/`.
  The contract's quantizer-level surface is still correct;
  centering is explicitly a task-27 AM concern per this finding.
- No re-run of the feasibility harness. A "residual encoding +
  beam search" test is an end-to-end system test — task 27's
  responsibility, not a quantizer-module test.

## Closing task 25

With this packet, task 25 (RaBitQ Stage 1) is **functionally
complete**:

- Slice 1–4: module scaffold + encoder + estimator (paper-faithful
  at slice 9).
- Slice 5–8: feasibility harness + null-result gate verdict (FAIL
  at 10 pp absolute; correctly framed).
- Slice 9: paper-faithful estimator port (the mechanical bug fix
  the slice-8 verdict flagged).
- Slice 10: re-run verdict after the math correction (FAIL holds
  at absolute encoding; confirms implementation integrity).
- Slice 11: `--rerank-k` harness flag proved K'=100 gives 1.00
  recall — validates the estimator on the non-Symphony rerank
  pipeline.
- Slice 12: q-bit encoder (parked as non-Symphony path, ADR
  follow-up).
- Slice 13: seed plumbing for prod hygiene.
- Slice 14 (this packet): Symphony-path finding; task 27 un-defer.

Task 27 starts next.
