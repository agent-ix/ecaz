# Review Request: IVF/RaBitQ Sidecar Rerank Bench

## Scope

Code commit: `ee876d09089dbc67a2faa824f7545e92227c3a8d`

This adds a suite-driven local measurement harness for Task 51 Exp 7:

- new `ecaz bench sidecar-rerank` command,
- new `ecaz bench suite` step kind `sidecar-rerank`,
- local benchmark packet `benchmarks/task51-local-ivf-rabitq-sidecar`.

The harness is deliberately measurement-only. It requires an isolated `ec_ivf` RaBitQ `rerank=off` table, fetches IVF approximate candidate ids, and locally reranks the candidate frontier through f32, f16, and bits=8 RaBitQ sidecar representations.

Important scope correction after reviewer feedback: this harness loads the source-sidecar representation into the CLI process before timed reranking. The `sidecar p50` and `total bound p50` columns are therefore a free-I/O upper bound for scoring/recall quality, not product sidecar storage latency. They do not model a real sidecar table, random id lookup, TID-sorted sidecar fetch, prefetch behavior, or an in-index read path.

## Local Benchmark Result

The local 50k sidecar suite completed successfully:

```text
completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

At `nprobe=64`, `candidate_k=50`, q=200:

| variant | recall@10 | candidate SQL p50 | sidecar p50 | total bound p50 | sidecar size |
| --- | ---: | ---: | ---: | ---: | ---: |
| f32 | 0.9940 | 107.509 ms | 1.552 ms | 109.450 ms | 292.97 MiB |
| f16 | 0.9940 | 107.509 ms | 2.561 ms | 110.184 ms | 146.48 MiB |
| rabitq8 | 0.9505 | 107.509 ms | 1.120 ms | 108.628 ms | 73.81 MiB |

Interpretation: on the current local rerank-off IVF scan, sidecar rerank CPU is small relative to approximate candidate acquisition when source bytes are already resident in the benchmark process. f32/f16 preserve recall for this candidate frontier; `rabitq8` is smaller but loses recall at width 50. These numbers answer recall-quality and scoring-CPU questions only; they are not a product sidecar latency forecast.

## Caveats

- Local PG18/WSL2 only; not AWS or Graviton evidence.
- No vchord or pgvectorscale was run.
- This is not a product sidecar implementation.
- Source-sidecar access is free in this harness. A real-I/O sidecar benchmark is still owed before making any product decision from Exp 7.
- `recall_p10 = 0.9000` at nprobe=32 is a candidate-frontier floor across variants; f32/f16 cannot recover neighbors that the `LIMIT 50` approximate frontier did not emit.
- `candidate SQL p50` is the IVF rerank-off query returning the top 50 approximate candidates to the client, not a full posting-frontier materialization.
- q=200 is a local preflight waiver; AWS promotion still needs the Task 51 q-count bar.

## Validation

- `cargo check -p ecaz-cli` passed.
- `cargo test -p ecaz-cli expands_sidecar_rerank_with_variants` passed.
- `git diff --check` passed.
- `ecaz bench suite` local sidecar run passed.

See `artifacts/manifest.md` for the packet-local artifact map.
