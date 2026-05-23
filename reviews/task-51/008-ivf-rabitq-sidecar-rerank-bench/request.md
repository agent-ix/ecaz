# Review Request: IVF/RaBitQ Sidecar Rerank Bench

## Scope

Code commit: `ee876d09089dbc67a2faa824f7545e92227c3a8d`

This adds a suite-driven local measurement harness for Task 51 Exp 7:

- new `ecaz bench sidecar-rerank` command,
- new `ecaz bench suite` step kind `sidecar-rerank`,
- local benchmark packet `benchmarks/task51-local-ivf-rabitq-sidecar`.

The harness is deliberately measurement-only. It requires an isolated `ec_ivf` RaBitQ `rerank=off` table, fetches IVF approximate candidate ids, and locally reranks the candidate frontier through f32, f16, and bits=8 RaBitQ sidecar representations.

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

Interpretation: on the current local rerank-off IVF scan, sidecar rerank CPU is small relative to approximate candidate acquisition. f32/f16 preserve recall for this candidate frontier; `rabitq8` is smaller but loses recall at width 50.

## Caveats

- Local PG18/WSL2 only; not AWS or Graviton evidence.
- No vchord or pgvectorscale was run.
- This is not a product sidecar implementation.
- q=200 is a local preflight waiver; AWS promotion still needs the Task 51 q-count bar.

## Validation

- `cargo check -p ecaz-cli` passed.
- `cargo test -p ecaz-cli expands_sidecar_rerank_with_variants` passed.
- `git diff --check` passed.
- `ecaz bench suite` local sidecar run passed.

See `artifacts/manifest.md` for the packet-local artifact map.
