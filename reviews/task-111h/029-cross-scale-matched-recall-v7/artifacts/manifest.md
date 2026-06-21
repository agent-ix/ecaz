# Artifact Manifest

Packet: `reviews/task-111h/029-cross-scale-matched-recall-v7`

Task bucket: `reviews/task-111h`

Head SHA before packet commit: `468c76b061e9bdb11882bdc5e9b9aa1ed17fab83`

Created: `2026-06-20`

## Scope

This packet is a derived analysis over existing Task 111h post-v7 benchmark
packets. No new benchmark suite was run.

It applies the same selection rule used by packet 025: for each recall target
and format, select the lowest-p50 row that reaches the target. If no row reaches
the target, report the maximum-recall row as `NO_REACH`.

Targets: `0.95`, `0.97`, `0.99`.

## Source Artifacts

| Scale | Packet | Result artifact | sha256 |
| --- | --- | --- | --- |
| 10k | `reviews/task-111h/026-rerank-suite-10k-v7` | `artifacts/results.jsonl` | `38f8b81d8fbe23d45cfd4befeef32d7079ef2c6c456eee3761652c14f70253f1` |
| 50k | `reviews/task-111h/027-rerank-suite-50k-v7` | `artifacts/results.jsonl` | `bc54a13b89e4823a48c8276684f333f98f83283c5046592f2132c602ce2ad040` |
| 100k | `reviews/task-111h/024-rerank-suite-100k-v7` | `artifacts/results.jsonl` | `768152e498a877cda4bc3483a2925040fec0837c043b80a52ffd687d1de30307` |
| 100k | `reviews/task-111h/024-rerank-suite-100k-v7` | `artifacts/results-rabitq4-cont-report.jsonl` | `e5474d61a5d9bf06f6b15c99c02f891cb4a294a0bc8310aa85b3da940ef07af5` |
| 100k | `reviews/task-111h/024-rerank-suite-100k-v7` | `artifacts/results-rabitq8-cont-report.jsonl` | `c1a13c1c74a025771536e769136b10fae17a3d4b05b80d22e710e6ece7a55280` |
| 100k | `reviews/task-111h/024-rerank-suite-100k-v7` | `artifacts/results-turboquant-cont-report.jsonl` | `c545ee7ec52d61ba7ea668f80574b698a4727a0a45c5a8dc0d7a51aa27b315e8` |
| 1M | `reviews/task-111h/028-rerank-suite-1m-v7-shared` | `artifacts/results.jsonl` | `9ff8cbf082f14197f32fe25ce4ed6330d0c563cafb12f7a91d203fd79926705e` |

The 100k packet uses continuation report JSONL for the compact quantized cells
because the main suite hit ENOSPC and the clean continuation runs are the
authoritative post-v7 compact-format evidence.

## Commands

10k:

```sh
jq -s -r -f reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/select-matched-recall.jq \
  reviews/task-111h/026-rerank-suite-10k-v7/artifacts/results.jsonl
```

50k:

```sh
jq -s -r -f reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/select-matched-recall.jq \
  reviews/task-111h/027-rerank-suite-50k-v7/artifacts/results.jsonl
```

100k:

```sh
jq -s -r -f reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/select-matched-recall.jq \
  reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results.jsonl \
  reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-rabitq4-cont-report.jsonl \
  reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-rabitq8-cont-report.jsonl \
  reviews/task-111h/024-rerank-suite-100k-v7/artifacts/results-turboquant-cont-report.jsonl
```

1M:

```sh
jq -s -r -f reviews/task-111h/029-cross-scale-matched-recall-v7/artifacts/select-matched-recall.jq \
  reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/results.jsonl
```

The derived table is recorded in
`artifacts/cross-scale-matched-recall-v7.md`.

## Surface Notes

- 10k/50k/100k: isolated one-index-per-table benchmark surfaces.
- 1M: shared-table benchmark surface with one active IVF index per cell, as
  documented in packet 028.

## Key Result Lines

- At 50k and target `0.95`, source/f32 is `0.9520` recall at `3.49 ms` p50 with
  a `13.8 MiB` IVF index; TurboQuant reaches `0.9550` at `5.47 ms` p50 with a
  `62.3 MiB` IVF index.
- At 100k and target `0.95`, source/f32 is `0.9625` recall at `6.23 ms` p50
  with a `24.6 MiB` IVF index; TurboQuant reaches `0.9530` at `11.6 ms` p50
  with a `104.4 MiB` IVF index.
- At 1M and target `0.95`, source/f32 is `0.9570` recall at `12.2 ms` p50 with
  a `226.8 MiB` IVF index; TurboQuant reaches `0.9510` at `42.4 ms` p50 with a
  `1013.9 MiB` IVF index.
- At target `0.99`, only source/f32 and f16 reach the target at 50k/100k/1M.
  f16 keeps source-like recall but uses much larger IVF indexes.

## Non-Claims

This packet does not close Task 111h. It does not add cold/remote evidence,
table-owned storage evidence, a legacy `0x2A` sidecar baseline, or new
correctness fixtures. It closes the cross-scale warm-cache matched-recall
decision-table gap only.
