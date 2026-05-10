# Request

Review the repeatable DiskANN prefilter benchmark surface and the attached PG18/M5 packet artifacts.

Scope:

- Adds `ec_diskann.rerank_budget` as a session GUC so benchmark sweeps can run `list_size == rerank_budget` without rebuilding one index per point.
- Adds generic `--set-guc` and `--set-guc-from-sweep` support to `ecaz bench recall`, `bench latency`, `compare pgvector`, `compare vectorscale`, and suite JSON steps.
- Adds `profile-diskann-prefilter-real10k.json` and wires it into `scripts/run_benchmark_profile.sh`.
- Measures the two implemented DiskANN traversal prefilters: `binary_sidecar` and `grouped_pq`.

What this checkpoint claims:

1. The DiskANN prefilter comparison is now repeatable from a committed suite rather than ad hoc SQL.
2. The new suite runs apples-apples `list_size == ec_diskann.rerank_budget` for ec_diskann and compares against pgvectorscale with `query_search_list_size == query_rescore`.
3. On real10k / 200 queries / k=10, `binary_sidecar` is the better implemented DiskANN prefilter. It is much higher recall than `grouped_pq` at low/mid widths with comparable latency.
4. pgvectorscale remains materially faster at the same search/rescore widths, despite similar index size.
5. True pgvectorscale-style SBQ is not an easy runtime-only switch for our current DiskANN payload: it requires a different persisted binary code and stored per-dimension threshold/mean metadata, so this checkpoint does not claim an SBQ implementation.

Key measured takeaways:

- `ec_diskann` binary sidecar:
  - `list_size=64`: `recall@10=0.9965`, latency `p50=2.17 ms`
  - `list_size=800`: `recall@10=1.0000`, latency `p50=15.7 ms`
- `ec_diskann` grouped PQ:
  - `list_size=64`: `recall@10=0.9320`, latency `p50=2.15 ms`
  - `list_size=800`: `recall@10=0.9990`, latency `p50=15.6 ms`
- pgvectorscale comparison:
  - `64`: pgvectorscale `recall@10=0.9955`, `p50=0.59 ms`; binary-sidecar ec_diskann `recall@10=0.9965`, `p50=2.15 ms`
  - `800`: pgvectorscale `recall@10=1.0000`, `p50=3.76 ms`; binary-sidecar ec_diskann `recall@10=1.0000`, `p50=15.4 ms`
- Storage:
  - ec_diskann index: `4.7 MiB` (`494.0 B/row`)
  - pgvectorscale DiskANN index: `5,136,384 bytes`

Packet-local evidence:

- [artifacts/manifest.md](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/manifest.md)
- [suite-manifest.json](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/suite-manifest.json)
- [results.jsonl](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/results.jsonl)
- [recall-diskann-binary-real10k.log](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/recall-diskann-binary-real10k.log)
- [recall-diskann-grouped-real10k.log](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/recall-diskann-grouped-real10k.log)
- [compare-vectorscale-binary-real10k.log](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/compare-vectorscale-binary-real10k.log)
- [compare-vectorscale-grouped-real10k.log](/Users/peter/dev/tqvector/review/30546-diskann-prefilter-benchmark-surface/artifacts/compare-vectorscale-grouped-real10k.log)
