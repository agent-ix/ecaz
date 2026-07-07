# Task 146 Packet 001: Shape Selection Preregistration

## Request

Please review the Task 146 Phase-0 shape selection and decision criteria before
the release Pareto matrix runs.

This packet starts Task 146 on branch
`task-146-spire-honest-pareto-confirmation`. It adds no new measurements. Its
purpose is to prevent post-hoc shape picking and to make the final promote /
iterate / shelve decision auditable.

## Inputs From Tasks 141-145

- Task 141 fixed the release substrate and rebaselined the anchor cells.
- Task 142 is closeable: the redundant per-query hierarchy reload floor is
  removed, including the nlists=2048 extension and epoch invalidation test.
- Task 143 is closeable: leaf-score-only routing is a positive release-validated
  candidate, but remains default-off because the measured grid covers only the
  current two-level exact-leaf shape. Route overfetch is not promoted.
- Task 144 is do-not-promote/escalate: closure/ratio pruning does not create a
  scalable <=5% scan high-recall operating point at 50k/100k.
- Task 145 is do-not-promote: no rerank-economy lever produced a held-recall
  latency win; bound-prune is provably inert/null on the remote path.

Faulty or non-engaged evidence remains rejected. In particular, Task 145 packet
008 latency/recall comparisons are not used here.

## Candidate Shapes

Run the same preregistered shapes on 10k, 50k, and 100k. If the 100k result has
a credible gate candidate, extend that shape to 1M in a later packet.

| ID | Purpose | Shape |
| --- | --- | --- |
| S1 | Historical anchor required by Task 146 | `nlists=128`, `boundary_replica_count=0`, defaults otherwise |
| S2 | Fine-grained release control | `nlists=1024`, `boundary_replica_count=0`, defaults otherwise |
| S3 | Fixed-count replication control | `nlists=1024`, `boundary_replica_count=2`, no closure/ratio pruning |
| S4 | Task 143 positive candidate | S2 plus `ec_spire.leaf_score_only_routing=on` |
| S5 | Task 143 combined-coverage check | S4 plus `ec_spire.route_overfetch_multiplier=2.0` |
| S6 | Router-saturation escalation candidate | S4 plus `top_graph_search_list_size=200` and `training_sample_rows=100000` where corpus size permits; use full corpus for 10k/50k if 100000 exceeds rows |

Common settings for all shapes:

- `source_identity=include`
- release `spire-local-multinode` substrate from Task 141
- standard nprobe sweep used by the program packets
- block-summary/pruning counters live in results and manifests
- no closure/ratio pruning from Task 144
- no bound-prune promotion or rerank-economy promotion from Task 145

## Matrix Scope

For each scale and shape:

- single-instance suite
- 3-worker multinode suite
- 200 or more queries
- recall, distinct recall, latency percentiles, storage, row-instance scan
  fraction, candidate/leaf/block counters
- suite manifest with per-node `ecaz_build_profile()`

## Preregistered Decision Criteria

Primary pass condition:

- `distinct_recall@10 >= 0.999`
- row-instances scanned `<= 15%` of the corpus-equivalent row instances
- p50 within `4x` the release IVF 100k anchor at matched recall, using
  Task 141/Task 76 anchors as the comparison source

Interpretation rules:

- If no shape satisfies recall plus scan fraction, Task 146 shelves or
  escalates the distributed SPIRE lane; it does not promote a latency-only
  point.
- If a shape satisfies recall/scan but misses the p50 factor only in multinode,
  report the single-instance vs multinode split and classify transport overhead
  separately. Do not call it a product Pareto win without the documented factor.
- If S6 is the only promising shape, the verdict is iterate/escalate rather
  than default promotion, because it represents router-saturation intervention
  rather than a validated default.
- Any mechanism whose engagement counter is zero is null evidence. It cannot
  support a promote or do-not-promote conclusion about that mechanism's value.

## Exclusions

- Do not rerun Task 139 cells.
- Do not include Task 145 bound-prune as a candidate unless a separate runtime
  fix first proves `pre_materialization_pruned_sum > 0` on the on-arm.
- Do not promote closure/ratio pruning from Task 144 into the matrix; carry only
  the do-not-promote/escalate conclusion.
- Do not add new mechanisms in Task 146.

