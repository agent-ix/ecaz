# Artifact Manifest: Task 111h Packet 041

Head SHA: `537c16ca82ed7e7808d19bff91151b0acb6e6465`

Task bucket: `reviews/task-111h/`

Packet path: `reviews/task-111h/041-final-closeout-decision/`

Timestamp: 2026-06-20 America/Los_Angeles

Lane: final local closeout audit over committed Task 111h code and packets.

Surface: no new benchmark surface. This packet cites previously committed isolated and shared suite surfaces:

- isolated one-prefix/table/index surfaces: packets `024`, `026`, `027`, `036`, `040`;
- shared-table one-active-index 1M surface: packet `028`;
- read-only derived analysis: packet `029`.

## Commands

This packet did not run new tests or benchmarks. The audit used read-only source and packet inspection:

```sh
nl -ba plan/tasks/111h-ivf-persisted-rerank-format-sweep.md
find reviews/task-111h -maxdepth 2 -type f -name request.md
find reviews/task-111h -maxdepth 3 -type f -path '*/feedback/*.md'
rg -n "RerankPlacement::Table|source_diagnostic|RerankPayloadCodec|EC_IVF_INDEX_FORMAT_VERSION|test_ec_ivf_index_placement_update_snapshot_payload" src tests docs fixtures
cat reviews/task-111h/{024,026,027,028,029,034,036,038,040}-*/request.md
```

The status update commit under review is:

```text
537c16ca82ed7e7808d19bff91151b0acb6e6465 task111h: mark persisted rerank sweep complete
```

## Artifact Inventory

| Artifact | Purpose | Result |
| --- | --- | --- |
| `artifacts/final-closeout-audit.md` | Requirement audit and final promote/iterate/abandon table. | Closes all Task 111h checklist and acceptance rows with packet-local evidence. |
| `artifacts/manifest.md` | Packet provenance and artifact metadata. | This file. |
| `request.md` | Review request for the closeout packet and tracker update. | Requests final review. |

## Key Result Lines Cited

- Packet `027`: 50k post-v7 suite status `completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- Packet `024`: 100k post-v7 suite plus continuation evidence; source/f32 reaches `0.9875-0.9990` recall@10 at nprobe 200, f16 `0.9870-0.9980`, RaBitQ4 `0.9330-0.9380`, RaBitQ8 `0.9455-0.9525`, TurboQuant `0.9525-0.9565`.
- Packet `028`: 1M v7 summary reports source/f32 w128 nprobe 200 recall `0.9910`, index/f16 w128 nprobe 200 recall `0.9910`, RaBitQ4 w128 `0.9370`, RaBitQ8 w128 `0.9520`, TurboQuant w128 `0.9510`.
- Packet `029`: matched-recall table shows source/f32 as the warm-cache local reference at 50k/100k/1M and f16 as recall-neutral but storage-heavy.
- Packet `036`: RaBitQ8 clip=4 at 100k reaches recall@10 `0.9915` to `0.9920` at nprobe 200 with `183.6 MiB` index size.
- Packet `040`: 50k cold diagnostic completed `46 failed=0`; source/f32 recall `0.9520/0.9875/0.9895`, f16 same recall but `172.5 MiB` IVF index, RaBitQ8 clip4 recall `0.9550/0.9915/0.9930`, TurboQuant `0.9300/0.9550/0.9565`.

## Non-Claims

- This packet does not add a new benchmark run.
- This packet does not claim remote-storage behavior.
- This packet does not promote compact index-side rerank as default.
