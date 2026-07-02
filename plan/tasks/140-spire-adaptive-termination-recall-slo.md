# Task 140: SPIRE Adaptive Termination Under An Explicit Recall SLO

Status: proposed (2026-07-02; filed from the Task 131 closeout research
synthesis). GATED: requires an explicit user/product recall-SLO decision
before any promotion work.
Owner: coder (to be assigned). One coder, one branch.
Priority: P2 — third act after Task 137 (correctness) and Task 139 (routing);
depends on Task 138's distinct-recall metric and Task 139's frontier shape.

## Why

Task 131 proved that *provably safe* scan-time pruning has no headroom on this
surface: the sound-bound skip ceiling collapsed from 5.4% of rows at 10k to
0.010% at 50k, because RaBitQ block bounds are too loose relative to the
kth-margin at scale. The safety proof, not the protocol timing, is the binding
constraint — the packet 027 diagnostic used the final global kth (the best
threshold any streaming protocol could ever converge to) and still found
nothing to skip.

The variant that escapes that ceiling drops the proof: stop probing when the
global top-k has *empirically stabilized*, accepting recall as a
high-probability target instead of a guarantee. At the current operating point
(nprobe=96 to reach recall 1.0), the marginal probe's hit rate is nearly zero
— recall saturates long before probing stops — which is exactly the regime
where stability-based termination wins.

Scaffolding already exists and is barely evaluated:

- `ec_spire.adaptive_nprobe` + `ec_spire.adaptive_nprobe_score_gap_micros`
  (Task 30 Phase 9): score-gap halving, measured only once
  (117.1 -> 115.9 ms local 10k — noise-level, never tuned, never distributed).
- Task 131 packet 015's candidate-to-heap streaming structure orders
  mid-scan candidate arrival, which a coordinator-side stability window can
  consume without any new worker protocol.

Task 131's non-goal ("do not use an approximate threshold as a recall-unsafe
drop rule") was correct for that task's contract. This task exists to make the
contract change explicit, opt-in, and measured — not to smuggle it in.

## Gate: Recall SLO Decision First

Before Phase 1 runs, the user must approve a written SLO statement in the
packet, e.g.: "adaptive mode targets distinct_recall@10 >= 0.995 (or 0.999)
with p99 recall floor X over the standard query sets; guaranteed-recall mode
remains the default." No promotion work without this sign-off. If the SLO is
declined, close the task as shelved-by-product-decision.

## Goal

Measure the latency saved by stability-based adaptive termination at fixed
distinct-recall targets (0.995 / 0.999 / 1.0-observed) on the Task 139
frontier shape, and decide promote-as-opt-in / iterate / shelve.

## Scope

### Phase 0 - Baseline The Existing Adaptive Path

- Evaluate `ec_spire.adaptive_nprobe` as-is (score-gap halving) on the
  frontier shape, single-instance and multi-instance, standard sweep:
  distinct-recall + latency vs fixed nprobe. It has never been measured
  beyond one local smoke.

### Phase 1 - Stability-Window Termination Design

- Coordinator-side: stop issuing/continuing list probes when the global
  top-k has not changed for the last M list-batches (M = the stability
  window; sweep it). Consume the packet-015 streaming candidate order; no
  coordinator->worker threshold protocol (Task 131 shelved that shape).
- Strict/degraded semantics, cancellation, pool reuse unchanged; fault
  drills on.
- Default-off GUC; guaranteed mode untouched.

### Phase 2 - Recall/Latency Tradeoff Matrix

- 10k/50k/100k on the frontier shape: stability window sweep x fixed-nprobe
  baseline, distinct_recall@10 + latency p50/p95/p99 + per-query recall
  distribution (an SLO needs tail recall, not just the mean).
- Skewed-worker fixture (reuse `--slow-candidate-node2-ms`) to prove
  termination composes with streaming under skew.

### Phase 3 - Decision

- Promote-as-opt-in only if: SLO met including tail, latency win beyond the
  measured noise floor at two scales, and fault behavior clean. Otherwise
  iterate or shelve with numbers.

## Required Evidence

- `ecaz bench suite`; per-query distinct-recall distributions packet-local;
  pre-registered success criteria before the Phase 2 matrix runs (Task 131
  packet 027 set the precedent).

## Non-Goals

- No revival of sound threshold feedback or bound metadata (Task 131).
- No change to the guaranteed-recall default path.
- No SLO invention by the coder — the gate decision belongs to the user.

## Acceptance Criteria

1. Written recall-SLO statement approved by the user before Phase 2.
2. Existing adaptive_nprobe path baselined with distinct-recall.
3. Stability-window termination measured at 10k/50k/100k with per-query
   recall distributions and a noise-floor-aware latency comparison.
4. Final packet: promote-as-opt-in / iterate / shelve, each with numbers.

## References

- `plan/tasks/138-spire-distinct-recall-metric-audit.md` (dependency)
- `plan/tasks/139-spire-routing-selectivity-pareto.md` (dependency: shape)
- `plan/tasks/131-spire-streaming-global-topk-pruning.md` (why sound pruning lost)
- `reviews/task-131/015-phase2-candidate-heap-streaming/` (streaming structure)
- `reviews/task-131/027-phase3-increment-a-ab/` (ceiling + noise floor + pre-registration precedent)
- `src/am/ec_spire/options/mod.rs` (`ec_spire.adaptive_nprobe*` GUCs)
