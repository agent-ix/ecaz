# Task 121 Packet 020 50k Recall Sampled Pruning Summary

## Scope

This packet extends packet 019's Phase 3 sampled-pruning pilot from 10k to the
50k scale, using the same b4/tr50/f8 RaBitQ block-summary candidate:

- `storage_format=rabitq`
- `leaf_block_rows=64`
- `leaf_block_summary_representatives=2`
- baseline pruning off
- sampled global pruning with `max_global_blocks=1024`,
  `global_probe_blocks=2048`, `sample_rows_per_block=4`,
  `sample_summary_prior_weight=0.8`, `summary_radius_weight=0.25`, and
  `route_prior_weight=0.0`

The selected suite was started for 50k and 100k, then intentionally stopped
after 50k storage plus paired 50k recall A/B completed. The full six-point
50k recall sweep was expensive enough that the latency, pipeline, and 100k
steps should be split into narrower follow-up suites.

## Storage

```text
index=203.4 MiB
index_per_row=4265.2 B
table_heap_toast_fsm_vm=793.8 MiB
total=998.3 MiB
total_per_row=20936.6 B
```

## Recall A/B

Sampled global pruning was faster at high nprobe, but it was not recall-neutral
at the saturated part of the curve. Off reaches recall 1.0000 by nprobe 48;
sampled remains at 0.9995 for nprobe 48/64/96.

| nprobe | off recall@10 | sampled recall@10 | off mean q-time | sampled mean q-time |
|---:|---:|---:|---:|---:|
| 8 | 0.9810 | 0.9810 | 371.82 ms | 379.34 ms |
| 16 | 0.9905 | 0.9905 | 603.99 ms | 614.80 ms |
| 32 | 0.9985 | 0.9985 | 1062.88 ms | 1040.89 ms |
| 48 | 1.0000 | 0.9995 | 1423.23 ms | 1115.16 ms |
| 64 | 1.0000 | 0.9995 | 1625.24 ms | 1200.44 ms |
| 96 | 1.0000 | 0.9995 | 2045.17 ms | 1261.73 ms |

The 50k truth-cache seed check also reached recall 1.0000 at nprobe 96 with
mean q-time 2011.35 ms.

## Interpretation

The 50k result changes the Phase 3 read of packet 019:

- The sampled global policy is a real scan-efficiency lever at 50k. At nprobe
  96 it cuts mean query time from 2045.17 ms to 1261.73 ms.
- The specific sampled setting is too aggressive for recall-preserving use at
  50k. It loses one recall trial out of 2000 at nprobe 48/64/96.
- The next scan-efficiency candidate should be a less aggressive 50k sampled
  setting, for example larger `max_global_blocks` / `global_probe_blocks` or
  more sample rows, before spending time on latency/pipeline/100k for this
  exact policy.

This packet still does not close Phase 3. It supplies 50k storage and recall
A/B only. 50k latency/pipeline plus 100k recall/latency/storage/pipeline remain
owed, and the default/TurboQuant block-summary surface is still not covered by
this RaBitQ-only pruning path.
