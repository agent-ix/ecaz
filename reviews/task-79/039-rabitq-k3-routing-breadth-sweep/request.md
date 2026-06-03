# Review Request: Task 79 RaBitQ K3 Routing-Breadth Sweep

## Summary

Packet 039 tests a direct candidate-work reduction axis after the packet 038 fast path: keep the best local k=3/global736 recipe, but lower routing breadth from `nprobe=96` to 64/72/80/88. This reduces routed leaves and object bytes, and it lowers p50 sharply.

Result: negative for Task 79 closure. Recall fails before the p50 gate is reached. At `nprobe=88`, p50 is close at 45.399 ms, but recall is only 0.9910. At `nprobe=96`, recall returns to 0.9925, but p50 is 47.936 ms. Candidate rows stay essentially flat at about 4.657M because the global736 block cap still fills from the routed leaf set.

## Evidence

- Packet manifest: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/manifest.md`
- Suite config: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/suite-rabitq-k3-routing-breadth-sweep.json`
- Compact results: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/compact-results.tsv`
- Raw suite output: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-run.log`
- Parsed results: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/results.jsonl`

```text
nprobe	route_sum	selected_pid_sum	candidates	object_bytes_sum	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
64	12800	12800	4657560	9285184652	38.277	44.475	0.9815	2000	fail_recall
72	14400	14400	4657439	10414027244	41.611	46.625	0.9865	2000	fail_recall
80	16000	16000	4657540	11542973836	43.430	51.223	0.9895	2000	fail_recall
88	17600	17600	4657349	12689352924	45.399	54.408	0.9910	2000	fail_recall_p50
96	19200	19200	4657668	13816992816	47.936	54.885	0.9925	2000	fail_p50
```

## Validation

- `ecaz bench suite audit` passed.
- `ecaz bench suite status` reports `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- The suite reused the packet 037 k=3 index and the packet 038 fast-path backend; no AWS was used.

## Reviewer Notes

Lowering routing breadth is not a viable strict closure path by itself. It reduces route/object-read work, but the selected row surface remains nearly fixed under `global736`, while the missing-routed-leaf recall loss appears before p50 reaches the gate. This suggests the next direct candidate-work path needs a different index shape or block selector, not just less top-graph breadth on the current shape.
