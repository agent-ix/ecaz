# Review Request: Task 121 Phase 3 100k Sampled-Pruning Retune

## Scope

This packet adds the 100k recall-retune gate for the Phase 3 sampled global
block-pruning policy. It builds the missing 100k b4/tr50/f8 RaBitQ
block-summary surface, captures storage, seeds packet-local ground truth, and
compares pruning off against the conservative sampled retune:

```text
max_global_blocks=4096
global_probe_blocks=8192
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

This is not a Phase 3 closeout packet. It answers whether the 50k retune scales
to 100k without recall loss before the longer 100k latency/pipeline A/B.

## Evidence

Packet manifest: `artifacts/manifest.md`

Suite status:

```text
[suite:task121-phase3-local-100k-sampled-retune] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Storage:

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

Truth-cache seed:

```text
nprobe=96 recall@10=1.0000 mean_q_time=4934.02 ms
```

Recall A/B:

```text
off:     recall@48=0.9945 recall@64=0.9985 recall@96=1.0000
retuned: recall@48=0.9945 recall@64=0.9985 recall@96=1.0000

off mean_q_time:     48=3482.10 ms 64=4090.92 ms 96=4681.82 ms
retuned mean_q_time: 48=3384.60 ms 64=3951.75 ms 96=4253.79 ms
```

## Read

The `g4096/p8192/r4` 100k retune is recall-neutral at nprobe 48/64/96. It also
reduces recall-harness mean query time at all three checkpoints, with the
largest win at nprobe 96.

This supports carrying the retuned sampled policy into the 100k
latency/pipeline A/B. It does not answer the packet-022 reviewer concern about
the low-nprobe operating point, and it does not provide TurboQuant block-summary
coverage. Those remain explicit Phase 3 follow-ups.

## Hygiene

The generated `artifacts/truth-cache-100k-q200-k10.json` is local-only and is
not part of this request. The committed evidence is the suite config, audit,
dry-run, run/status logs, suite manifests/results JSONL, storage log, truth-cache
log, recall logs, and summary.
