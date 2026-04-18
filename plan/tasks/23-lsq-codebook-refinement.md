# Task 23: LSQ Codebook Refinement (Drop-In k-means Replacement)

Status: proposed — low priority, cheap fill-in work.

Executes ADR-038.

## Scope

Replace the k-means inner loop in grouped PQ codebook training with
Local Search Quantization (LSQ). Iterated local search refines
codebook assignments past the k-means fixed point; adds +2–5% recall
at the same byte budget.

**No wire format change.** Codes still fit in the same bits; the bits
just name slightly-better codebook entries.

This task is the least urgent entry in the post-task-17 queue. It is
here because it is cheap to implement, orthogonal to everything else,
and the measurement infrastructure already exists.

## Design outline

See ADR-038 for details. Summary:

- **k-means finds a local optimum** over assignment + centroid update.
  LSQ adds a third operator: swap candidate codebook entries between
  nearby subvectors if the total quantization error drops.
- **Convergence.** 5–10 LSQ passes after k-means converges typically
  captures the bulk of the +2–5% recall gain. Diminishing returns
  after that.
- **Runtime.** Build-time impact is proportional (~2x build time per
  codebook). Query-time: zero. Scan-time: zero. Just a training-side
  trick.

## Subtasks

- [ ] **LSQ kernel.** `src/quant/lsq.rs`. Takes the output of grouped
  k-means (codebooks + assignments) and runs `N` iterations of
  candidate swapping. Terminates on convergence or iteration budget.
- [ ] **Training pipeline hook.** Add an optional LSQ pass after
  `grouped_pq_train(...)` convergence. Gate behind a build-time
  setting (`lsq_iterations`, default `0`). Zero iterations = current
  behavior.
- [ ] **A/B harness.** Same seed, same training sample, compare:
  - k-means-only codebook
  - k-means + LSQ-5 codebook
  - k-means + LSQ-20 codebook
  on the 50k warm real seam. Measure recall@10 at the same
  `ef_search` budget.
- [ ] **Decision.** If recall delta ≥ +2 pp, flip default to
  `lsq_iterations = 5`. If < +2 pp, keep opt-in.
- [ ] **Determinism.** LSQ must be deterministic under fixed seed.
  Same rule that already governs grouped k-means per task 15.
- [ ] **Docs.** README note on the `lsq_iterations` knob.

## Owns

- ADR-038
- `src/quant/lsq.rs` (new)

## Dependencies

- Any task that trains grouped PQ codebooks. Task 15 (PqFastScan
  first-class) is sufficient; it composes with task 20 (OPQ) if
  that's landed first.

## Unblocks

- Small but free recall uplift on any grouped-PQ-based format
  (PqFastScan, future OPQ, future RVQ).

## Out of scope

- AQ / RVQ (task 22). LSQ refines *existing* codebooks; it does not
  change what kind of codebook is trained.
- OPQ rotation (task 20).
- Scoring kernel changes.

## Notes

- **Low ceiling, low floor.** +2–5% recall is a modest win; the
  point is it's nearly free. Do not spend extra effort beyond the
  implementation budget documented here.
- **Build time is the only cost.** +2x codebook training time is
  invisible for one-shot builds and annoying during rapid iteration.
  Default-off unless measurement clearly supports default-on.
- **Ordering flexibility.** Can slot in whenever there's a gap in
  the queue; no hard dependency chain after task 15.
- **Not a research task.** If LSQ doesn't show +2 pp on our seam,
  record the result and move on. Unlike task 22, this one doesn't
  warrant a research-track degradation path.
