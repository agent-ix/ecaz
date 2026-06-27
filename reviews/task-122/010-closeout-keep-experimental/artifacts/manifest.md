# Task 122 Packet 010 Artifact Manifest

- head SHA: `8c76506da452a02524cdaa314cb4a128746dbfd3`
- task bucket: `reviews/task-122/010-closeout-keep-experimental`
- timestamp: `2026-06-27T14:55:58Z`
- scope: closeout synthesis, no new benchmark run
- outcome: keep experimental / promote follow-up

## Inputs

This packet synthesizes existing Task 122 evidence:

- `reviews/task-122/001-tq-scorer-inventory/`
- `reviews/task-122/005-spire-prune-release-suite/`
- `reviews/task-122/006-spire-recall-width-sweep/`
- `reviews/task-122/007-spire-latency-storage-width25/`
- `reviews/task-122/009-sidecar-tq-stage2-suite/`

## Artifacts

- `request.md`: closeout request and decision.
- `artifacts/manifest.md`: this manifest.

No new test, benchmark, corpus, truth-cache, or generated data artifacts are
created in this packet.

## Key Cited Evidence

Packet 005:

```text
Candidate materialization drops from:
10k: 251,555 to 8,495
50k: 525,067 to 11,796
100k: 766,494 to 10,517
```

Packet 007:

```text
TQ and RaBitQ are latency-equivalent and storage-equivalent in this SPIRE width-25 matrix.
```

Packet 009:

```text
scale  nprobe  f32 recall/p95      TQ->f32@25 recall/p95
10k    32      1.0000 / 2.389 ms   1.0000 / 1.855 ms
10k    64      1.0000 / 2.898 ms   1.0000 / 2.176 ms
50k    32      0.9960 / 4.822 ms   0.9960 / 3.885 ms
50k    64      1.0000 / 7.835 ms   1.0000 / 6.819 ms
100k   32      0.9730 / 8.713 ms   0.9730 / 7.953 ms
100k   64      1.0000 / 13.815 ms  1.0000 / 13.517 ms
```
