# Task 51 Local IVF Adaptive Nprobe Smoke

This benchmark packet records a local-only PG18 smoke suite for the opt-in
`ec_ivf` adaptive nprobe policy on the preserved 990k IVF/RaBitQ surface:

- `ec_ivf`
- `storage_format=rabitq`
- `quant_bits=1`
- `nlists=1024`
- static/adaptive `nprobe` sweep `64,128,256`
- `rerank=heap_f32`
- `rerank_width=50`

Suite status: `completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

Main finding: the default aggressive threshold (`gap=1000`) buys some local
p50 reduction but loses recall and recall tail, so it is not production-ready
as-is. Conservative thresholds (`10000`, `100000`) preserve recall on this
q=100 smoke but do not show a material latency win.

This packet does not use AWS, vchord, or pgvectorscale.
