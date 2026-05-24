# Review Request: Local IVF/RaBitQ Scale And Counters

- task: 51
- packet: `reviews/task-51/007-local-ivf-rabitq-scale-counters`
- benchmark packet: `benchmarks/task51-local-ivf-rabitq-scale`
- benchmark commit: `00cfaf0e9` (`Add Task 51 local IVF RaBitQ scale suite`)
- scope: benchmark evidence only; no code change
- AWS: not used
- vchord: not run
- pgvectorscale: not run

## Summary

This checkpoint packages the local 50k/100k IVF/RaBitQ scale suite requested
by reviewer feedback on packet 006. It carries `nlists=64` and `nlists=128`
forward, adds sub-knee nprobe cells, and records representative EXPLAIN
counters.

The suite completed:

```text
[suite:task51-local-ivf-rabitq-scale] completed=20 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Findings

At 50k, `nlists=128` gives a real matched-recall local win:

| Cell | Recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: |
| `n64,p64` | `0.9975` | `190.2 ms` | `214.7 ms` |
| `n128,p96` | `0.9975` | `138.5 ms` | `162.6 ms` |

At 100k, the high-recall result does not clear the 25% gate:

| Cell | Recall@10 | p50 | p95 |
| --- | ---: | ---: | ---: |
| `n64,p64` | `0.9985` | `379.5 ms` | `419.3 ms` |
| `n128,p128` | `0.9985` | `377.7 ms` | `418.0 ms` |

Representative counters at `nprobe=64` show the expected scan-volume drop:

| Cell | Postings scored | Approx scan | Exact rerank |
| --- | ---: | ---: | ---: |
| 50k `n64,p64` | 50000 | `184.3 ms` | `2.2 ms` |
| 50k `n128,p64` | 23505 | `87.4 ms` | `2.5 ms` |
| 100k `n64,p64` | 100000 | `376.2 ms` | `2.0 ms` |
| 100k `n128,p64` | 48351 | `223.3 ms` | `3.2 ms` |

## Review Notes

This packet explicitly does not claim Graviton v4 promotion evidence. It is
local WSL2 evidence and does not measure NEON byte-LUT behavior. The q-count
is 200 with a local-screen waiver; AWS must still use q-count >= 500 unless
separately waived.

## Files Changed

- `benchmarks/task51-local-ivf-rabitq-scale/suite.json`
- `benchmarks/task51-local-ivf-rabitq-scale/manifest.md`
- `benchmarks/task51-local-ivf-rabitq-scale/request.md`
- `benchmarks/task51-local-ivf-rabitq-scale/artifacts/*`

## Validation

- `ecaz bench suite audit`: passed
- `ecaz bench suite run --dry-run`: passed
- `ecaz bench suite run`: passed
- `ecaz bench suite status`: `completed=20 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `git diff --check`: passed

See `artifacts/manifest.md` for packet-local review metadata.
