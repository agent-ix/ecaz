# Task 65b Packet 017 Artifact Manifest

- head SHA: `44e8d9ef5cc2fb1fbbb63d22533e6504aa5a34ec`
- task bucket: `reviews/task-65b/017-worker-zero-task65-head`
- timestamp: `2026-06-05T22:25:07Z`
- lane: local PG18, `ec_diskann`, `pq_fastscan`
- Task 65 head commit: `e195d4daf5f18d04dd9539c4f0a63f91f996b3bb`
- current branch comparison source: packet 008 plus fresh real10k rerun
- isolation: one index per table prefix

## Commands And Artifacts

Task 65 head setup:

- `git worktree add .task-worktrees/task65-head e195d4daf5f18d04dd9539c4f0a63f91f996b3bb`
- `install-task65-head.log`: installed Task 65-head backend SHA
  `ae4e57c70a9b74662cadd1872045afbcbe928b1f3e086bf41d3bdf596502e81b`.

Task 65 head real10k:

- `drop-task65head-real10k.log`: dropped `task65head_w0_real10k` tables before rebuild.
- `load-task65head-real10k-w0.log`: built
  `task65head_w0_real10k_pq_fastscan_idx` in `8.03s`.
- `graph-task65head-real10k-w0.log`: graph summary under the Task 65 extension;
  digest rows are `<missing>` because Task 65 predates the digest renderer.
- `graph-task65head-real10k-w0-current-summary.log`: graph summary after
  reinstalling the current extension, with digest rows:
  - `live_node_tid_digest=b476ea9f9a43d92eff12389fab3a013060d0a1cfdc47665af859194b4764d1bd`
  - `adjacency_digest=af9fe980fb9d0f6149d4102a82d561af0fc7e9b2fde422f47acc5e1e3cf7f0b5`
  - `first_256_node_digest=da8ab263ef126cffc5e62ddd42969e86f58b75e860f8b87f1327649246e2a667`

Task 65 head real100k:

- `drop-task65head-real100k.log`: dropped `task65head_w0_real100k` tables before rebuild.
- `load-task65head-real100k-w0.log`: built
  `task65head_w0_real100k_pq_fastscan_idx` in `243.32s`.
- `graph-task65head-real100k-w0-current-summary.log`: graph summary after
  reinstalling the current extension, with digest rows:
  - `live_node_tid_digest=5739d9a6040ccf6fe041e297d201a5a25537d18955398d9054c378926d81de53`
  - `adjacency_digest=683af2fb14938b475054f2d735d14e89a162947e93dba795d0077c5f492b5a12`
  - `first_256_node_digest=e332f9a4cba1318e4563adc9e2802d33ffefd161be3c76abf14eed503c31b4f7`

Current branch restore and fresh real10k rerun:

- `install-current-after-task65-head.log`: restored current extension backend SHA
  `e27af2d65111fa41c40b9b0e9843353522d730b96f22ed776464c506b987d7dd`.
- `drop-current-real10k-w0-rerun.log`: dropped `task65b_w0_rerun_real10k`
  tables before rebuild.
- `load-current-real10k-w0-rerun.log`: built
  `task65b_w0_rerun_real10k_pq_fastscan_idx` in `6.49s`; timing NOTICE shows
  `parallel_requested_workers=0`, `parallel_effective_workers=0`, and
  `parallel_rayon_scaffold=0`.
- `graph-current-real10k-w0-rerun.log`: current-branch digest rows match the
  Task 65 head real10k digest rows exactly.

## Comparison

| corpus | criterion | Task 65 head | current branch | status |
|---|---|---:|---:|---|
| real10k | build time | `8.03s` | `6.49s` | passes 5% tolerance |
| real10k | live TID digest | `b476...d1bd` | `b476...d1bd` | byte-equal |
| real10k | adjacency digest | `af9f...f0b5` | `af9f...f0b5` | byte-equal |
| real10k | first-256 digest | `da8a...a667` | `da8a...a667` | byte-equal |
| real100k | build time | `243.32s` | `243.15s` | passes 5% tolerance |
| real100k | live TID digest | `5739...de53` | `5739...de53` | byte-equal |
| real100k | adjacency digest | `683a...5a12` | `683a...5a12` | byte-equal |
| real100k | first-256 digest | `e332...b4f7` | `e332...b4f7` | byte-equal |

The current branch real100k comparison uses packet 008's current-branch
worker-zero row; this packet adds the missing Task 65-head baseline for that
same digest surface.
