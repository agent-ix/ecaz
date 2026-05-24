# task51-local-ivf-rabitq-geometry

This packet records a local-only IVF/RaBitQ geometry sweep for Task 51.

No AWS capacity was used. No vchord or pgvectorscale measurements were run.
The benchmark matrix was driven by `ecaz bench suite` using checked-in
`suite.json`; the final run produced `artifacts/suite-manifest.json` and
`artifacts/results.jsonl`.

## Scope

- local PG18 scratch only
- DBpedia real 10k fixture
- `ec_ivf`
- `storage_format=rabitq`
- `quant_bits=1`
- `rerank=heap_f32`
- `rerank_width=50`
- `nlists={32,64,128}`

## Result

The suite completed cleanly:

```text
[suite:task51-local-ivf-rabitq-geometry] completed=12 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Headline local cells:

| Geometry | Recall cell | Latency p50 | Index size |
| --- | ---: | ---: | ---: |
| `nlists=32,nprobe=8` | recall@10 `0.9985` | `13.5 ms` | `3.3 MiB` |
| `nlists=64,nprobe=8` | recall@10 `0.9970` | `7.91 ms` | `3.6 MiB` |
| `nlists=128,nprobe=16` | recall@10 `0.9970` | `7.95 ms` | `4.4 MiB` |

`nlists=128,nprobe=8` is the fastest low-nprobe cell at `5.52 ms` p50, but
its recall@10 is lower at `0.9935`.

## Follow-up

Use this packet to choose the larger local IVF/RaBitQ suite before the AWS
final gate. The practical next cells are `nlists=64` and `nlists=128` on a
larger local fixture; AWS should stay off until local scale-up and any code
changes are done.

See `manifest.md` for full command and artifact provenance.

