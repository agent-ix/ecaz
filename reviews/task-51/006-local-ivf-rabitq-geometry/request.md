# Review Request: Local IVF/RaBitQ Geometry Suite

- task: 51
- packet: `reviews/task-51/006-local-ivf-rabitq-geometry`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-geometry`
- benchmark commit: `cb140d6be` (`Add Task 51 local IVF RaBitQ geometry suite`)
- scope: benchmark evidence only; no code change
- AWS: not used
- vchord: not run
- pgvectorscale: not run

## Summary

This checkpoint packages the first local Task 51 IVF/RaBitQ-only experiment.
It runs a suite-driven `nlists` geometry sweep on the local DBpedia 10k
fixture:

- `storage_format=rabitq`
- `quant_bits=1`
- `rerank=heap_f32`
- `rerank_width=50`
- `nlists={32,64,128}`

The final suite run completed all 12 steps and produced structured
`suite-manifest.json` and `results.jsonl` outputs in the benchmark packet.

## Headline Results

| Geometry | Recall cell | Latency p50 | Index size |
| --- | ---: | ---: | ---: |
| `nlists=32,nprobe=8` | recall@10 `0.9985` | `13.5 ms` | `3.3 MiB` |
| `nlists=64,nprobe=8` | recall@10 `0.9970` | `7.91 ms` | `3.6 MiB` |
| `nlists=128,nprobe=16` | recall@10 `0.9970` | `7.95 ms` | `4.4 MiB` |

The local 10k result favors carrying `nlists=64` and `nlists=128` into a
larger local IVF/RaBitQ suite before any AWS final gate.

## Files Changed

- `benchmarks/task51-local-ivf-rabitq-geometry/suite.json`
- `benchmarks/task51-local-ivf-rabitq-geometry/manifest.md`
- `benchmarks/task51-local-ivf-rabitq-geometry/request.md`
- `benchmarks/task51-local-ivf-rabitq-geometry/artifacts/*`

## Validation

- `ecaz bench suite audit`: passed
- `ecaz bench suite dry-run`: passed
- `ecaz bench suite run`: passed
- `ecaz bench suite status`: `completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

See `artifacts/manifest.md` for packet-local review metadata and
`benchmarks/task51-local-ivf-rabitq-geometry/manifest.md` for benchmark
provenance.
