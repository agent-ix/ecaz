# Task 121 Packet 021 50k Sampled Pruning Retune Summary

## Scope

Packet 020 showed that the first 50k sampled global pruning policy was too
aggressive:

```text
g1024/p2048/r4: r@48=0.9995 r@64=0.9995 r@96=0.9995
```

This packet tests one conservative retune on the same 50k b4/tr50/f8 RaBitQ
block-summary index from packet 020:

```text
ec_spire.leaf_block_pruning_max_global_blocks=2048
ec_spire.leaf_block_pruning_global_probe_blocks=4096
ec_spire.leaf_block_pruning_sample_rows_per_block=4
ec_spire.leaf_block_pruning_sample_summary_prior_weight=0.8
ec_spire.leaf_block_pruning_summary_radius_weight=0.25
ec_spire.leaf_block_pruning_route_prior_weight=0.0
```

## Result

The retuned policy recovers recall 1.0000 at all saturated checkpoints:

| nprobe | off recall@10 from packet 020 | g1024/p2048/r4 from packet 020 | g2048/p4096/r4 retune | off mean q-time | retune mean q-time |
|---:|---:|---:|---:|---:|---:|
| 48 | 1.0000 | 0.9995 | 1.0000 | 1423.23 ms | 1421.46 ms |
| 64 | 1.0000 | 0.9995 | 1.0000 | 1625.24 ms | 1605.34 ms |
| 96 | 1.0000 | 0.9995 | 1.0000 | 2045.17 ms | 1786.64 ms |

The retune keeps a smaller high-nprobe speedup than the aggressive packet 020
policy, but it closes the recall loss:

```text
nprobe 96 mean q-time:
off            2045.17 ms
g1024/p2048/r4 1261.73 ms, recall 0.9995
g2048/p4096/r4 1786.64 ms, recall 1.0000
```

## Interpretation

The 50k policy candidate to carry forward is `g2048/p4096/r4`, not the packet
020 `g1024/p2048/r4` setting. It is recall-neutral at the exact checkpoints
where off was saturated, while still reducing nprobe 96 mean query time by
258.53 ms versus pruning off.

This is still recall-only evidence. The next required Phase 3 step is latency
and pipeline A/B for this retuned policy, followed by 100k recall/latency/
storage/pipeline evidence.
