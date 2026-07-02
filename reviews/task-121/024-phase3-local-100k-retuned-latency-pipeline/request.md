# Review Request: Task 121 Phase 3 100k Retuned Latency/Pipeline

## Scope

This packet adds the 100k latency, pipeline, storage, and recall evidence for
the Phase 3 sampled global block-pruning retune:

```text
max_global_blocks=4096
global_probe_blocks=8192
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

It reuses the packet 023 100k b4/tr50/f8 RaBitQ block-summary surface and
compares pruning off against the retuned sampled policy at `nprobe=8,16,32,48,96`.

This is not a Phase 3 or task closeout packet. It addresses the packet-022
request for 100k pipeline/latency coverage and the operating-point question.
TurboQuant block-summary coverage and the final Phase 4 Pareto/verdict remain
open.

## Evidence

Packet manifest: `artifacts/manifest.md`

Suite status:

```text
[suite:task121-phase3-local-100k-retuned-latency-pipeline] completed=7 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Storage:

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=4639.98 ms
```

Clean latency p50:

```text
off:     8=960.5 ms 16=1624.1 ms 32=2678.3 ms 48=3411.2 ms 96=4622.4 ms
retuned: 8=951.4 ms 16=1641.3 ms 32=2685.4 ms 48=3367.9 ms 96=4200.8 ms
```

Pipeline p50 and recall:

```text
off p50:     8=954.048 ms 16=1603.581 ms 32=2618.912 ms 48=3368.828 ms 96=4607.716 ms
retuned p50: 8=946.735 ms 16=1602.378 ms 32=2616.362 ms 48=3372.773 ms 96=4211.934 ms

off recall:     8=0.9330 16=0.9670 32=0.9895 48=0.9945 96=1.0000
retuned recall: 8=0.9330 16=0.9670 32=0.9895 48=0.9945 96=1.0000
```

Pipeline counters:

```text
object bytes unchanged at every nprobe:
8=5504487114 16=11144593728 32=22297157442 48=33073013046 96=64389908578

nprobe=96 candidates: off=76623116 retuned=56982159
nprobe=96 heap_rerank: off=19307246 retuned=17473898
```

## Read

The retuned sampled policy is recall-neutral across the 100k sweep, but the
operating-point answer is narrow: it does not materially improve low `nprobe`
settings (`8..32`), and the `nprobe=48` clean-latency hint is flat in the
pipeline harness. The real win is at `nprobe=96`, where pipeline p50 improves
8.6% with unchanged recall.

The mechanism is post-read compute/rerank reduction, not I/O reduction. Object
bytes are unchanged at all checkpoints; candidate and heap-rerank counts only
fall at `nprobe=96`.

## Hygiene

The generated `artifacts/truth-cache-100k-q200-k10.json` is local-only and is
not part of this request. The committed evidence is the suite config, audit,
dry-run, run/status logs, suite manifests/results JSONL, storage log,
truth-cache log, latency logs, pipeline logs, and summary.
