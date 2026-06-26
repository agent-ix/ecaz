# Task 89 Review Request: IVF TQ+ Projected QJL Suite

## Summary

This checkpoint adds suite-driven fixture plumbing for a reachable IVF
TurboQuant QJL/gamma lane and records a projected DBPedia 10k A/B suite.

Code changes under review:

- `ecaz corpus load --input-dim` validates staged TSV rows at their on-disk
  dimension and stores only the first `--dim` values. This lets suite loads
  reuse staged DBPedia without committing derived projected TSVs.
- `ecaz bench recall --truth-input-dim` projects local truth TSV rows to the
  loaded query dimension before computing exact truth.
- `ecaz bench suite` passes both fields through load and recall steps.

Benchmark evidence under review:

- `suite.json` projects staged DBPedia 10k from 1536 to 768 dimensions.
- 768 dimensions forces the generic TurboQuant QJL/gamma path because the
  no-QJL tiled 4-bit path is only enabled for the 1536-dimensional compatibility
  tile.
- The suite compares baseline TurboQuant and TQ+ with load, recall@10, latency,
  and storage steps.

## Result

The projected QJL/gamma lane does not rescue the IVF TQ+ profile. At the
representative `nprobe=48` cell:

| variant | recall@10 | mean q-time | latency p50 | latency p95 | index bytes/row |
| --- | ---: | ---: | ---: | ---: | ---: |
| TQ baseline | 0.9070 | 1.75 ms | 1.76 ms | 1.86 ms | 535.8 B |
| TQ+ | 0.9100 | 8.41 ms | 8.42 ms | 8.91 ms | 536.6 B |

TQ+ gains 0.3 recall points but is about 4.8x slower at p50 and p95 on this
projected 768-dimensional QJL/gamma path. Storage is effectively unchanged.

Across the full sweep:

- Baseline recall@10: `0.9060`, `0.9070`, `0.9070`, `0.9070` at
  `nprobe=16,32,48,64`.
- TQ+ recall@10: `0.9110`, `0.9100`, `0.9100`, `0.9100`.
- Baseline latency p50: `0.76 ms`, `1.22 ms`, `1.76 ms`, `2.29 ms`.
- TQ+ latency p50: `3.11 ms`, `5.61 ms`, `8.42 ms`, `11.1 ms`.

Load timing was also mildly worse for TQ+: baseline total load was `16.47 s`
with `0.288 s` index build; TQ+ total load was `16.58 s` with `0.520 s` index
build.

## Validation

Completed in this packet:

- `cargo test -p ecaz-cli` passed: 412 tests.
- `./target/debug/ecaz bench suite audit --config reviews/task-89/004-ivf-tqplus-qjl-projected-suite/suite.json`
  passed: 8 steps.
- `./target/debug/ecaz bench suite run --config reviews/task-89/004-ivf-tqplus-qjl-projected-suite/suite.json --dry-run --manifest-output reviews/task-89/004-ivf-tqplus-qjl-projected-suite/artifacts/suite-manifest-dry-run.json`
  expanded the projected load and recall commands.
- DBPedia projected-QJL 10k suite run completed through `ecaz bench suite` and
  wrote `artifacts/suite/results.jsonl`.

## Not Claimed

This is not Task 89 closeout evidence.

Open gates after this packet:

- Insert/update drift evidence.
- At least one non-DBPedia corpus.
- Public-shape gate and closeout decision.
