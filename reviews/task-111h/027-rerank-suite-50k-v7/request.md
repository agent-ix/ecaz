# Review Request: Task 111h 50k V7 Rerank Suite

## Scope

This packet adds a post-v7 50k rerank-format/width sweep driven by
`ecaz bench suite`. It extends the current benchmark evidence between the
existing 10k and 100k packets for source-side f32 and index-side f16,
RaBitQ4, RaBitQ8, and TurboQuant.

No production code changes are under review in this packet.

## Evidence

Packet manifest:

- `artifacts/manifest.md`

Primary structured artifacts:

- `artifacts/task111h-50k-rerank-format-width-v7-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/results-report.jsonl`
- `artifacts/summary-nprobe32.md`
- `artifacts/suite-status.log`

Suite status:

```text
[suite:task111h-50k-rerank-format-width-v7] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Fastest row per variant at recall >= 0.95, plus RaBitQ4 at max observed
recall because it did not reach 0.95:

| Variant | Width | nprobe | Recall@10 | p50 | p99 | Index size | Note |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| source/f32 | 32 | 32 | 0.9520 | 3.49 ms | 4.34 ms | 13.8 MiB | >=0.95 recall |
| index/f16 | 32 | 32 | 0.9520 | 3.63 ms | 5.32 ms | 172.5 MiB | >=0.95 recall |
| index/turboquant | 32 | 128 | 0.9550 | 5.47 ms | 6.87 ms | 62.3 MiB | >=0.95 recall |
| index/rabitq8 | 64 | 128 | 0.9520 | 6.21 ms | 7.96 ms | 93.4 MiB | >=0.95 recall |
| index/rabitq4 | 128 | 200 | 0.9460 | 8.86 ms | 11.0 ms | 54.0 MiB | max recall; below 0.95 |

Full nprobe32, nprobe200, and storage tables:
`artifacts/summary-nprobe32.md`.

## Readout

- The 50k warm-cache local run does not support the earlier f16 150 ms p50
  claim. The worst f16 p50 in this matrix is 19.9 ms
  (`index/f16`, width 256, nprobe128).
- At matched recall >= 0.95, source-side f32 is slightly faster than
  index-side f16 and keeps the ec_ivf index at 13.8 MiB. Index-side f16 at the
  same width/recall uses 172.5 MiB.
- TurboQuant is the best compact index-side row in this packet: width 32 at
  nprobe128 reaches 0.9550 recall with p50 5.47 ms and 62.3 MiB index storage.
- RaBitQ8 reaches 0.9520 recall at width 64/nprobe128 but is slower and larger
  than TurboQuant in this run.
- RaBitQ4 tops out at 0.9460 recall in this 50k fixture.

## Review Ask

Please review the packet for benchmark provenance, artifact completeness, and
whether the interpretation above is appropriately limited to the 50k
warm-cache fixture.
