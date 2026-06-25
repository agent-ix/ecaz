# Task 121 Phase 3 100k Sampled-Pruning Retune Summary

Packet: `reviews/task-121/023-phase3-local-100k-sampled-retune/`

## Scope

This packet builds the missing 100k b4/tr50/f8 RaBitQ block-summary surface and
checks whether the conservative sampled global block-pruning retune remains
recall-neutral before spending the longer 100k latency/pipeline run.

Index shape:

- `nlists=128`
- `recursive_fanout=8`
- `top_graph_enabled=1`
- `top_graph_degree=32`
- `top_graph_build_list_size=100`
- `top_graph_search_list_size=96`
- `boundary_replica_count=4`
- `training_sample_rows=50000`
- `storage_format=rabitq`
- `ec_spire.leaf_block_rows=64`
- `ec_spire.leaf_block_summary_representatives=2`

Retuned sampled policy:

- `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`
- `ec_spire.leaf_block_pruning_max_global_blocks=4096`
- `ec_spire.leaf_block_pruning_global_probe_blocks=8192`
- `ec_spire.leaf_block_pruning_sample_rows_per_block=4`
- `ec_spire.leaf_block_pruning_sample_summary_prior_weight=0.8`
- `ec_spire.leaf_block_pruning_summary_radius_weight=0.25`
- `ec_spire.leaf_block_pruning_route_prior_weight=0.0`

## Storage

From `storage-100k_b4_tr50_f8_block64.log`:

```text
index=404.8 MiB, index_per_row=4244.8 B
total=1.9 GiB, total_per_row=20915.2 B
```

## Recall

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

The 100k `g4096/p8192/r4` retune is recall-neutral across the high-nprobe
checkpoints and reduces recall-harness mean query time by 97.50 ms at nprobe 48,
139.17 ms at nprobe 64, and 428.03 ms at nprobe 96.

## Interpretation

This closes the 100k recall-retune gate for the sampled global pruning policy,
but not the full Phase 3 A/B closeout. The packet does not include standalone
latency or pipeline/object-byte counters. Per reviewer feedback on packet 022,
the remaining Phase 3 work must still show the 100k latency/pipeline A/B,
confront the shipped operating point question, and either run or explicitly
decide the TurboQuant block-summary coverage.
