# Task 51 Local IVF/RaBitQ 990k Current-Shape Measurement

This benchmark packet records a local-only PG18 IVF/RaBitQ suite on the 990k anchor corpus using the current Task 51 shape:

- `ec_ivf`
- `storage_format=rabitq`
- `quant_bits=1`
- `nlists=1024`
- `nprobe` sweep `64,96,128,192,256`
- `rerank=heap_f32`
- `rerank_width=50`

Suite status: `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

The strongest local recall point is `nprobe=256`: recall@10 `0.9850`, NDCG@10 `0.9995`, latency p50 `1083.8 ms`, p95 `1197.9 ms`.

The knee-like local point is around `nprobe=128`: recall@10 `0.9750`, NDCG@10 `0.9986`, latency p50 `566.0 ms`, p95 `659.8 ms`.

Storage: the `ec_ivf` RaBitQ index is `298.3 MiB` for `990000` rows, `316.0 B` per row. The table is `15.4 GiB` because this local surface preserves the source vectors for truth/rerank checks.

This packet does not use AWS, vchord, or pgvectorscale. It is the local smoke/scale gate before any AWS final gate.
