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
