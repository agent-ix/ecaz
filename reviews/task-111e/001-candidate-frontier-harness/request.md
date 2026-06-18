# Task 111e: Candidate Frontier Measurement Slice

## Summary

This packet starts Task 111e Phase 1 by measuring dense RaBitQ-1 candidate
frontier containment with exact f32 oracle rerank on the real 50k fixture.

The code change makes the existing `ecaz bench sidecar-rerank` harness emit the
frontier diagnostics that the coarse-rerank Phase 1 needs:

- `candidate_count_min`, `candidate_count_p50`, `candidate_count_p95`
- `sidecar_bytes_touched_p50`, `sidecar_bytes_touched_p95`

The existing harness already asks an isolated `rerank=off` IVF/RaBitQ index for
coarse candidates and then reranks those ids with a configurable sidecar
representation. For Task 111e, this is the first measurement surface for dense
RaBitQ-1 candidate containment before implementing a new AM query mode.

## Code Under Review

- `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`

The code change is intentionally measurement-only. It does not change index
storage, scan behavior, scoring, or built-in rerank behavior.

## Packet Evidence

Artifacts are under `reviews/task-111e/001-candidate-frontier-harness/artifacts/`.

- `cargo-test-ecaz-cli-sidecar-rerank.log`: focused package test run passed
  (`4 passed; 0 failed; 404 filtered out`).
- `task111e-candidate-frontier-suite.json`: first 50k Phase 1 suite config.
- `suite-audit.log`: suite audit passed (`7 steps`).
- `create-task111e-db.log`: created the local PG18 measurement database.
- `suite/load-50k-rb1-dense-page-rerank-off.log`: loaded the 50k dense
  RaBitQ-1 page-local `rerank=off` fixture.
- `suite-run-sidecar-pg18-rebuilt.log`: authoritative rebuilt-binary
  sidecar-rerank run.
- `suite/results.jsonl` and `suite-report.log`: parsed result rows.

## Suite Shape

The packet-local suite creates a fresh 50k dense RaBitQ-1, page-local,
`rerank=off` fixture:

- `storage_format=rabitq`
- `quant_bits=1`
- `dense_posting_blocks=1`
- `dense_posting_pack_pages=1`
- `dense_posting_typed_layout=1`
- `rerank=off`
- `nprobe=32`

It then runs f32 oracle rerank over candidate_k:

`25`, `50`, `100`, `256`, `512`, `1000`

## Result Summary

50k real corpus, 100 queries, nprobe 32, f32/free oracle rerank:

| candidate_k | recall@10 | NDCG@10 | candidate SQL p50 | sidecar score p50 | total bound p50 | bytes touched p50 |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 25 | 0.9750 | 0.9995 | 7.536 ms | 0.761 ms | 8.308 ms | 150.00 KiB |
| 50 | 0.9940 | 0.9997 | 8.586 ms | 1.522 ms | 10.105 ms | 300.00 KiB |
| 100 | 0.9960 | 0.9997 | 13.316 ms | 4.419 ms | 17.835 ms | 600.00 KiB |
| 256 | 0.9960 | 0.9997 | 17.658 ms | 7.974 ms | 25.886 ms | 1.50 MiB |
| 512 | 0.9960 | 0.9997 | 27.850 ms | 16.125 ms | 44.257 ms | 3.00 MiB |
| 1000 | 0.9960 | 0.9997 | 48.340 ms | 31.171 ms | 79.696 ms | 5.86 MiB |

The candidate frontier does not need thousands of candidates on this 50k cell:
candidate_k 50 already reaches `0.9940` recall@10 and candidate_k 100 reaches
the observed plateau at `0.9960`. Wider frontiers only add latency and f32 bytes
touched in this run.

## Validation

```text
cargo test -p ecaz-cli sidecar_rerank
4 passed; 0 failed; 404 filtered out

target/debug/ecaz bench suite audit --config reviews/task-111e/001-candidate-frontier-harness/artifacts/task111e-candidate-frontier-suite.json
audit passed: 7 steps

target/debug/ecaz bench suite run --config reviews/task-111e/001-candidate-frontier-harness/artifacts/task111e-candidate-frontier-suite.json --artifact-dir reviews/task-111e/001-candidate-frontier-harness/artifacts/suite --database task111e_coarse_rerank --host /home/peter/.pgrx --port 28818 --only ...sidecar steps...
completed=6 failed=0 skipped=1 dry_run=0 missing_artifacts=0 stale=0
```

## Review Ask

Please review whether the 50k candidate-frontier result is credible enough to
proceed to the next Task 111e slice: 100k containment plus the heap-f32
`coarse_rerank` contract/reloption design.
