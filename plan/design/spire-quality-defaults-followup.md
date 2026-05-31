# SPIRE Quality Defaults Follow-Up

Task 73 showed that SPIRE's stock 100k setting is not a hard recall ceiling:
`top_graph_search_list_size=128`, `boundary_replica_count=0`, and higher
`nprobe` reached `0.9975` to `1.0000` recall@10 on the measured 100k fixture.
The same evidence also showed that the quality point is much slower than the
current default and materially slower than the IVF control at matched recall.

This is a product/default policy question, not a Task 73 tuning slice:

- keep the current fast default and document a quality preset;
- change the default toward the high-recall setting and accept the latency
  cost;
- add adaptive/profile-driven defaults keyed by corpus size and recall target.

A future defaults task should start from:

- local M5 quality packet:
  `reviews/task-73/001-spire-m5-quality-gate/`
- AWS confirmation packet:
  `benchmarks/task73-74-aws-spire-quality-overhead/`
- Task 74 profiler packet once accepted, because it explains where the high
  recall point spends time.

No default should change until that task has same-host recall, latency, and
profiler evidence for the proposed policy.

## Task 76 Update

Task 76 ran the local Intel `ecaz bench suite` Pareto packet at 10k and 100k:
`benchmarks/task76-intel-local-spire-pareto/`.

The result amends this follow-up decision without changing SPIRE defaults:

- 10k has cheap high-recall SPIRE points, but 100k remains the governing case.
- At 100k, SPIRE tg96/nprobe96 reached recall@10 `0.9975` with p50
  `146.693 ms` and p95 `175.128 ms`, while IVF nprobe96 reached recall@10
  `0.9980` with p50 `37.7 ms` and p95 `46.5 ms`.
- The canonical local 1M TSV fixture was unavailable, so Task 76 does not
  promote a 1M-informed default or quality preset.

If this policy is reopened, it should start from Task 75's candidate-funnel
evidence plus Task 76's Pareto packet and should prioritize reducing SPIRE's
candidate/materialization cost before raising default recall aggression.
