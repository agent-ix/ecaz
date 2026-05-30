# Task 32 Packet 001 Docs Follow-Up

Reviewer: please review this metadata/docs follow-up for the approved Task 32
M5 DiskANN cross-engine refresh.

## Scope

This addresses the non-blocking reviewer notes from
`reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/feedback/2026-05-30-01-reviewer.md`.

Changes:

- Rewrites packet `001`'s `artifacts/manifest.md` from scaffold text into the
  packet-local source of truth: head SHA, surface, commands, artifact index, key
  result lines, and instrumentation notes.
- Adds exact `pg_relation_size` evidence for the four compared indexes in
  `artifacts/index-size-bytes.sql.log`.
- Appends four `kind=summary` rows to packet `001`'s `artifacts/results.jsonl`
  so downstream docs can consume one row per engine with build time and exact
  index bytes.
- Updates `docs/benchmarks.md` to publish the final post-M5 Task 32 row and to
  call out the local M5 latency/build gap versus `pgvectorscale` explicitly.

## Key Result Lines

At matched low tuning on real10K warm cache:

| engine | tuning | recall@10 | mean | p99 | build | index size |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `ec_diskann` | `64` | `0.9965` | `2.14 ms` | `2.67 ms` | `9.84 s` | `4,939,776 B` |
| `pgvectorscale` | `64` | `0.9960` | `0.60 ms` | `0.88 ms` | `1.48 s` | `5,136,384 B` |

The packet now states that memory HWM was not measured: the latency commands
requested backend memory sampling, but every row emitted `memory_samples=0`.

## Validation

- `jq empty reviews/task-32/001-30210-task32-m5-diskann-final-cross-engine-refresh/artifacts/results.jsonl`
- `git diff --check`

Tests were not run because this is a docs/artifact metadata follow-up with no
runtime code changes.

## Review Focus

- Is packet `001`'s manifest now sufficient as the source of truth for docs?
- Are the added summary rows in `results.jsonl` clear enough for downstream
  benchmark table extraction?
- Does `docs/benchmarks.md` state the `pgvectorscale` gap plainly without
  overgeneralizing beyond local M5 evidence?
