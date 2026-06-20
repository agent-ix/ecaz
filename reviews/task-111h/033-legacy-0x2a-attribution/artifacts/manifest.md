# Artifact Manifest

Packet: `reviews/task-111h/033-legacy-0x2a-attribution`

Task bucket: `reviews/task-111h`

Current head audited:
`28a5cf08fce3d2f233e296111cbbf597e84775a3`

Created: `2026-06-20`

## Scope

This packet closes the Task 111h legacy-baseline evidence row by citing and
summarizing the existing direct-TID `0x2A` benchmark packet:

`reviews/task-111g/005-direct-sidecar-rerank-tids`

No new benchmark was run for this packet. Current v7 HEAD intentionally cannot
rerun `0x2A`: it writes packed `0x2B`/`0x2C` rerank groups and rejects v4
metadata.

## Artifact Index

| Artifact | Description |
| --- | --- |
| `legacy-0x2a-attribution.md` | Read-only audit and attribution report tying Task 111h's legacy baseline requirement to the completed 111g/005 `ecaz bench suite` artifacts. |

## Cited External Packet Artifacts

The benchmark evidence cited by this packet is packet-local under
`reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/`:

| Artifact | Description |
| --- | --- |
| `manifest.md` | Source of truth for the direct-TID suite metadata, head SHA, commands, result lines, and prior ADR-079 comparison. |
| `suite-status.log` | Suite completion status: `completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`. |
| `sidecar-index-direct-tids/suite-config.json` | Packet-local suite config. |
| `sidecar-index-direct-tids/suite-manifest.json` | Completed suite manifest. |
| `sidecar-index-direct-tids/results.jsonl` | Normalized recall, latency, storage, and load results. |
| `sidecar-index-direct-tids/{latency,recall,storage,load}-*.log` | Raw per-step logs. |

## Commands

No new benchmark commands were run for this packet. The cited 111g/005 packet
records the original `ecaz bench suite` commands in its manifest.

Read-only audit commands used to prepare this packet:

```sh
rg -n "0x2A|rerank_sidecar|legacy" src/am/ec_ivf docs plan reviews/task-111h -S
git grep -n "append_ivf_rerank_sidecar_block_to_new_block|IvfRerankSidecarBlockTuple::new" HEAD -- src/am/ec_ivf
git grep -n "build_rerank_group_chain|IvfRerankGroupHeaderTuple" HEAD -- src/am/ec_ivf
rg -n '"step":"latency-f16idx-100k"|"step":"latency-rq4idx-100k"|"step":"storage-f16idx-100k"|"step":"storage-rq4idx-100k"|"step":"recall-f16idx-100k"|"step":"recall-rq4idx-100k"' reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/sidecar-index-direct-tids/results.jsonl
```

## Key Result Lines

From `reviews/task-111g/005-direct-sidecar-rerank-tids/artifacts/suite-status.log`:

```text
[suite:ivf-attr-sidecar-index-placement] completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

From the 111g/005 manifest and results:

```text
f16 index 100k, direct TID packet: p50 2.99 ms / 6.02 ms / 13.0 ms at nprobe 8 / 64 / 200
rabitq4 index 100k, direct TID packet: p50 2.79 ms / 5.72 ms / 11.9 ms at nprobe 8 / 64 / 200
f16 index size 100k: 416.6 MiB
rabitq4 index size 100k: 103.6 MiB
```

## Non-Claims

This packet does not close table-owned persisted compact payload storage,
RaBitQ-4/RaBitQ-8 slab cleanup or benchmark-away evidence, cold/remote
benchmark evidence, or the final Task 111h decision table.
