# Review Request: Task 121 Phase 4 Final Pareto/Verdict

## Scope

This packet is the Task 121 Phase 4 synthesis. It adds no new benchmark run; it
ties the committed Phase 1/2/3 packets into the required cost/quality verdict.

Reviewer sign-off is requested for Task 121 closeout.

## Verdict

Do not promote a new SPIRE default from Task 121.

Task 121 proved the Task 120 failure mode: the lossy stage is route/leaf
selection, not candidate/rerank. It also proved the main recovery lever:
boundary replication. But the Pareto is not good enough for a default. The
route-stage truth loss can be bought back, but the cost slope is boundary
replica storage plus low-nprobe latency. Phase 3 scan-efficiency only helps at
high `nprobe`; it does not make the likely low operating point cheap.

Best named follow-up candidate:

```text
storage_format=rabitq
boundary_replica_count=4
training_sample_rows=50000
recursive_fanout=8
nlists=128
top_graph_search_list_size=96
leaf_block_rows=64
leaf_block_summary_representatives=2
```

Use retuned sampled block pruning only for high-recall `nprobe=96` experiments:

```text
max_global_blocks=4096
global_probe_blocks=8192
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

Do not make that pruning policy a broad default: it is neutral at low nprobe
and increases memory.

## Route-Stage Finding

The Stage 1 re-plan packet recorded the decisive attribution:

```text
route-stage containment equals final recall in every completed run.
The route stage is still the bottleneck.
```

The significant set was therefore route-focused:

```text
boundary_replica_count: primary
training_sample_rows=50000: secondary
nlists=316: interaction/cost axis only
TurboQuant: compatibility/control, not route recovery
```

That satisfies the task's finding-tied acceptance criterion: the final
recommendation is based on levers that address the route-stage loss.

## Pareto Evidence

Selected local evidence:

```text
10k, nprobe=4:
b0/tr10/f8  recall=0.9810 p50=18.657 ms index=9.4 MiB
b1/tr50/f8  recall=0.9900 p50=25.964 ms index=17.2-17.3 MiB
b4/tr50/f8  recall=0.9935 p50=44.714 ms index=40.6-40.7 MiB

50k, b4/tr50/f8:
nprobe=8  recall=0.9810 p50=400.503 ms
nprobe=16 recall=0.9905 p50=658.855 ms
nprobe=32 recall=0.9985 p50=1130.192 ms
nprobe=48 recall=1.0000 p50=1482.835 ms
index=196.9 MiB, index_per_row=4128.8 B

100k, b4/tr50/f8:
nprobe=8  recall=0.9330 p50=955.0 ms clean
nprobe=16 recall=0.9670 p50=1629.8 ms clean
nprobe=32 recall=0.9895 p50=2699.4 ms clean
nprobe=48 recall=0.9945 p50=3451.9 ms clean
nprobe=96 recall=1.0000 p50=4730.6 ms clean
index=392.2-392.3 MiB, index_per_row=4112-4113 B

100k, b8 wall:
b8 reaches recall 1.0000 by nprobe=64
b8 clean p50@96=5215.1-5283.8 ms
b8 index=704.7-704.8 MiB, index_per_row=7389-7390 B
```

Read: b4/tr50/f8 is the practical recall-recovery knee. b8 proves saturation
but is a wall, not a default: it buys the last recall points with a much larger
index and slower clean latency.

## Phase 3 Scan-Efficiency Evidence

The RaBitQ block-summary retune is recall-neutral after retuning, but the
benefit is high-nprobe only:

```text
10k sampled pruning:
nprobe=96 pipeline p50 342.133 ms -> 283.740 ms
candidates 7463419 -> 5121349
object bytes unchanged

50k retuned sampled pruning:
nprobe=48 pipeline p50 1409.952 ms -> 1384.971 ms, recall 1.0000 -> 1.0000
nprobe=64 pipeline p50 1666.490 ms -> 1548.382 ms, recall 1.0000 -> 1.0000
nprobe=96 pipeline p50 2073.081 ms -> 1737.456 ms, recall 1.0000 -> 1.0000
object bytes unchanged

100k retuned sampled pruning:
nprobe=8  pipeline p50 954.048 ms -> 946.735 ms, recall 0.9330 -> 0.9330
nprobe=16 pipeline p50 1603.581 ms -> 1602.378 ms, recall 0.9670 -> 0.9670
nprobe=32 pipeline p50 2618.912 ms -> 2616.362 ms, recall 0.9895 -> 0.9895
nprobe=48 pipeline p50 3368.828 ms -> 3372.773 ms, recall 0.9945 -> 0.9945
nprobe=96 pipeline p50 4607.716 ms -> 4211.934 ms, recall 1.0000 -> 1.0000
```

At 100k, the retuned policy only reduces candidate/heap work at `nprobe=96`:

```text
nprobe=96 candidates: off=76623116 retuned=56982159
nprobe=96 heap_rerank: off=19307246 retuned=17473898
object bytes unchanged at all nprobe values
```

Read: Phase 3 does not solve the default operating point. It is useful only as
a high-recall compute/rerank optimization. It does not reduce I/O in the local
pipeline counters.

## TurboQuant Decision

TurboQuant was route/recall neutral in Stage 1:

```text
baseline recall@10:   0.7250 0.8525 0.9045 0.9310 0.9645 0.9825 0.9975
turboquant recall@10: 0.7250 0.8525 0.9045 0.9310 0.9645 0.9825 0.9975
```

Packet 025 records the implementation gap: scan-side global/sample block
pruning is still RaBitQ-gated. Since RaBitQ pruning did not become a default
candidate, implementing and measuring TurboQuant pruning inside Task 121 would
not change this verdict.

## Final Read

Task 121 changed the conclusion from "coarse routing cannot work" to a more
precise one: routing precision can be recovered locally, but the current
recovery lever is too expensive. The wall is not candidate quantization, block
selection, or rerank width. The wall is route recall versus boundary-replica
cost at the low/mid operating point.

Close Task 121 as an evidence-backed no-promote result. Carry `b4/tr50/f8` as
the named follow-up candidate only for future non-default experiments, especially
if a later task introduces a cheaper route-precision mechanism than boundary
replication.

## Evidence

Packet manifest: `artifacts/manifest.md`
