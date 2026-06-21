# Review Request: Task 111h 10k V7 Rerank Suite

## Scope

This packet adds a post-v7 10k rerank-format/width sweep driven by
`ecaz bench suite`. It reruns the old 10k pilot shape against the current
residual RaBitQ and centroid-relative TurboQuant code paths.

No production code changes are under review in this packet.

## Evidence

Packet manifest:

- `artifacts/manifest.md`

Primary structured artifacts:

- `artifacts/task111h-10k-rerank-format-width-v7-suite.json`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/results-report.jsonl`
- `artifacts/summary-nprobe32.md`
- `artifacts/suite-status.log`

Suite status:

```text
[suite:task111h-10k-rerank-format-width-v7] completed=81 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Fastest nprobe32 row per format:

| placement | format | width | recall32 | p50 | p99 | recall200 | index size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9970 | 2.84 ms | 3.50 ms | 0.9985 | 5.1 MiB |
| index | f16 | 32 | 0.9960 | 2.02 ms | 2.73 ms | 0.9975 | 37.0 MiB |
| index | rabitq4 | 32 | 0.9765 | 1.75 ms | 2.30 ms | 0.9780 | 14.7 MiB |
| index | rabitq8 | 32 | 0.9840 | 1.85 ms | 2.59 ms | 0.9855 | 22.3 MiB |
| index | turboquant | 32 | 0.9795 | 2.01 ms | 2.55 ms | 0.9810 | 14.7 MiB |

Full nprobe32 width table: `artifacts/summary-nprobe32.md`.

## Readout

- The 10k warm-cache local run does not support the earlier f16 150 ms claim.
  Index-side f16 width 32 measured p50 2.02 ms and p99 2.73 ms at nprobe32.
- At this small scale, index-side f16 is faster than source-side f32 at similar
  recall, but it uses about 37.0 MiB of index storage versus 5.1 MiB for
  source-side f32.
- RaBitQ4, RaBitQ8, and TurboQuant have the fastest nprobe32 rows, but their
  nprobe200 recall ceilings stay below source-side f32 and index-side f16 on
  this fixture.
- This is scale-specific evidence only. The 100k post-v7 packet remains the
  better current signal for the larger local warm-cache tradeoff.

## Review Ask

Please review the packet for benchmark provenance, artifact completeness, and
whether the interpretation above is appropriately limited to the 10k warm-cache
fixture.
