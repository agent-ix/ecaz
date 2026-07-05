# Task 65b Packet 017: Worker-Zero Task 65 Head Baseline

This packet addresses packet 008 reviewer feedback: the worker-zero fallback
packet needed a digestable Task 65 head baseline and a fresh real10k rebuild
on the current branch.

## Method

- Created an isolated git worktree at Task 65 close commit
  `e195d4daf5f18d04dd9539c4f0a63f91f996b3bb`.
- Installed the Task 65 extension build from that worktree.
- Built worker-zero real10k and real100k DiskANN indexes with the same corpus,
  seed, storage format, and `graph_degree=32`, `build_list_size=100`,
  `alpha=1.2` settings used by packet 008.
- Reinstalled the current Task 65b extension so the newer
  `ec_diskann_index_graph_summary()` digest renderer could compute comparable
  digest rows for the Task 65-built indexes.
- Rebuilt a fresh current-branch worker-zero real10k index to close the packet
  008 skipped-rebuild timing gap.

## Results

### Real10k

| build | build time | live TID digest | adjacency digest | first-256 digest |
|---|---:|---|---|---|
| Task 65 head | `8.03s` | `b476ea9f9a43d92eff12389fab3a013060d0a1cfdc47665af859194b4764d1bd` | `af9fe980fb9d0f6149d4102a82d561af0fc7e9b2fde422f47acc5e1e3cf7f0b5` | `da8ab263ef126cffc5e62ddd42969e86f58b75e860f8b87f1327649246e2a667` |
| Current branch fresh rerun | `6.49s` | `b476ea9f9a43d92eff12389fab3a013060d0a1cfdc47665af859194b4764d1bd` | `af9fe980fb9d0f6149d4102a82d561af0fc7e9b2fde422f47acc5e1e3cf7f0b5` | `da8ab263ef126cffc5e62ddd42969e86f58b75e860f8b87f1327649246e2a667` |

The current branch is byte-equal by all three graph digests and faster than
Task 65 head on this rerun.

### Real100k

| build | build time | live TID digest | adjacency digest | first-256 digest |
|---|---:|---|---|---|
| Task 65 head | `243.32s` | `5739d9a6040ccf6fe041e297d201a5a25537d18955398d9054c378926d81de53` | `683af2fb14938b475054f2d735d14e89a162947e93dba795d0077c5f492b5a12` | `e332f9a4cba1318e4563adc9e2802d33ffefd161be3c76abf14eed503c31b4f7` |
| Current branch packet 008 | `243.15s` | `5739d9a6040ccf6fe041e297d201a5a25537d18955398d9054c378926d81de53` | `683af2fb14938b475054f2d735d14e89a162947e93dba795d0077c5f492b5a12` | `e332f9a4cba1318e4563adc9e2802d33ffefd161be3c76abf14eed503c31b4f7` |

The current branch is byte-equal by all three graph digests and within the
5% timing tolerance versus Task 65 head (`243.15s` vs `243.32s`).

## Evidence

Packet-local artifact metadata is in `artifacts/manifest.md`.

This packet should close packet 008's worker-zero byte-equality and real10k
fresh-timing asks.
