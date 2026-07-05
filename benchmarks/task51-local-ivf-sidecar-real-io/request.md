# Benchmark Request: Local IVF Sidecar Real-I/O Smoke

## Scope

Benchmark packet:

- `benchmarks/task51-local-ivf-sidecar-real-io/`

Code commit measured:

- `0b359e5ddbee42a7cba45042f7da577d1accf7d4` - real-I/O sidecar rerank modes

This local PG18 suite reuses the preserved 50k isolated IVF/RaBitQ
`rerank=off` surface from packet 008. It does not rebuild the corpus, does not
use AWS, and does not run vchord or pgvectorscale.

## Result

Suite status:

```text
completed=1 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main finding: sidecar storage shape matters.

- Random primary-key lookup adds about 17-18 ms p50 sidecar I/O for 50
  candidates.
- TID-sorted batch fetch adds about 0.9-1.4 ms p50 sidecar I/O.
- F16 preserves the candidate-frontier recall in this fixture and reaches
  recall@10 `0.9980` by nprobe 96/128.
- RaBitQ8 is smaller but stays recall-limited at about `0.9470-0.9480`.

## Decision

The packet 008 free-I/O sidecar numbers should not be treated as a product
forecast by themselves. This real-I/O smoke says a naive random-id sidecar is
not promising, while a batched physical-order sidecar read remains plausible
enough to carry into the final Pareto decision.

See `manifest.md` for artifact details and exact commands.
