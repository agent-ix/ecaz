# task51-local-ivf-rabitq-scale

This benchmark packet records the local IVF/RaBitQ scale-up after reviewer
feedback on the 10k geometry screen.

No AWS capacity was used. No vchord or pgvectorscale measurements were run.
The matrix was driven by checked-in `ecaz bench suite` config and produced
structured `suite-manifest.json` plus `results.jsonl`.

## Scope

- local PG18 scratch only
- DBpedia real 50k and 100k fixtures
- `ec_ivf`
- `storage_format=rabitq`
- `quant_bits=1`
- `rerank=heap_f32`
- `rerank_width=50`
- `nlists={64,128}`
- q-count 200, with local-screen waiver documented in `manifest.md`

## Result

The suite completed cleanly:

```text
[suite:task51-local-ivf-rabitq-scale] completed=20 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

The local 50k result favors `nlists=128`: at matched recall@10 `0.9975`,
`nlists=128,nprobe=96` is `138.5 ms` p50 versus `190.2 ms` for
`nlists=64,nprobe=64`.

The local 100k result is less decisive: at recall@10 `0.9985`,
`nlists=128,nprobe=128` is `377.7 ms` p50 versus `379.5 ms` for
`nlists=64,nprobe=64`. At a lower recall band, `nlists=128,nprobe=96`
is `290.3 ms` p50 at recall@10 `0.9970`.

Representative EXPLAIN counters show approximate scan dominates exact rerank:
for 100k `nlists=128,nprobe=64`, approximate scan is `223.3 ms` while exact
rerank is `3.2 ms`.

## Follow-up

Do not promote only one geometry to AWS from this local packet. Carry both
`nlists=64` and `nlists=128` into the final gate or any 1M local/AWS frontier
packet, and sequence Experiment 7 sidecar measurements before AWS per reviewer
feedback.

See `manifest.md` for full provenance and caveats.

