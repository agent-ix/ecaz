# Review Request: Local IVF/RaBitQ 990k Current-Shape Suite

Please review the local-only Task 51 IVF/RaBitQ 990k measurement packet:

- benchmark packet: `benchmarks/task51-local-ivf-rabitq-990k/`
- SuiteConfig: `benchmarks/task51-local-ivf-rabitq-990k/suite.json`
- benchmark manifest: `benchmarks/task51-local-ivf-rabitq-990k/manifest.md`
- packet-local report: `reviews/task-51/009-local-ivf-rabitq-990k-current-shape/artifacts/suite-report.log`
- packet-local status: `reviews/task-51/009-local-ivf-rabitq-990k-current-shape/artifacts/suite-status.log`

## Scope

This is a measurement-only packet. No code changed in this packet.

The suite is IVF/RaBitQ only:

- `ec_ivf`
- `storage_format=rabitq`
- `quant_bits=1`
- `nlists=1024`
- `nprobe` sweep `64,96,128,192,256`
- `rerank=heap_f32`
- `rerank_width=50`

AWS was not used. vchord and pgvectorscale were not run.

## Result

Suite status:

```text
[suite:task51-local-ivf-rabitq-990k] completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Key local results:

- load succeeded: total `2420.91s`, index build `1671.85s`
- recall@10 q=100:
  - nprobe 64: `0.9570`, mean q-time `320.75 ms`
  - nprobe 96: `0.9690`, mean q-time `419.90 ms`
  - nprobe 128: `0.9750`, mean q-time `561.32 ms`
  - nprobe 192: `0.9820`, mean q-time `822.94 ms`
  - nprobe 256: `0.9850`, mean q-time `1103.46 ms`
- latency q=100/concurrency=1:
  - nprobe 64: p50 `285.2 ms`, p95 `351.5 ms`
  - nprobe 128: p50 `566.0 ms`, p95 `659.8 ms`
  - nprobe 256: p50 `1083.8 ms`, p95 `1197.9 ms`
- storage: `298.3 MiB` ec_ivf RaBitQ index, `316.0 B` per row
- explain counters show heap rerank width capped at `50` for both p128 and p256

## Caveats

- Local PG18/WSL2 only; not Graviton.
- Recall uses q=100 as a local cost waiver.
- The source corpus comes from the staged 990k anchor corpus and uses `--allow-manifest-mismatch` only because this packet creates a new isolated Task 51 prefix from that staged source.
- `quant_bits` still emits the loader profile-registry warning, but the storage and explain outputs confirm the index reloptions include `quant_bits=1`.
