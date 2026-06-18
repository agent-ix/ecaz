# Task 111e: Candidate Frontier Harness Slice

## Summary

This packet starts Task 111e by making the existing `ecaz bench sidecar-rerank`
harness emit the frontier diagnostics that the coarse-rerank Phase 1 needs:

- `candidate_count_min`, `candidate_count_p50`, `candidate_count_p95`
- `sidecar_bytes_touched_p50`, `sidecar_bytes_touched_p95`

The existing harness already asks an isolated `rerank=off` IVF/RaBitQ index for
coarse candidates and then reranks those ids with a configurable sidecar
representation. For Task 111e, that is the right first measurement surface for
dense RaBitQ-1 candidate containment before implementing a new AM query mode.

## Code Under Review

- `crates/ecaz-cli/src/commands/bench/sidecar_rerank.rs`

The code change is intentionally measurement-only. It does not change index
storage, scan behavior, scoring, or built-in rerank behavior.

## Packet Evidence

Artifacts are under `reviews/task-111e/001-candidate-frontier-harness/artifacts/`.

- `cargo-test-ecaz-cli-sidecar-rerank.log`: focused package test run passed
  (`4 passed; 0 failed; 404 filtered out`).
- `task111e-candidate-frontier-suite.json`: first 50k Phase 1 suite config.
- `suite-dry-run.log` and `suite/suite-manifest.json`: `ecaz bench suite`
  dry-run expanded the intended load and candidate_k sweep commands.

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

This is a harness/config checkpoint, not a final measurement packet. The next
Task 111e slice should run this suite, capture the recall/NDCG/latency rows,
and then add the 100k cell after regenerating or restoring the 100k corpus
fixture locally.

## Validation

```text
cargo test -p ecaz-cli sidecar_rerank
4 passed; 0 failed; 404 filtered out

target/debug/ecaz bench suite run --dry-run --config reviews/task-111e/001-candidate-frontier-harness/artifacts/task111e-candidate-frontier-suite.json --artifact-dir reviews/task-111e/001-candidate-frontier-harness/artifacts/suite --database task111e_coarse_rerank --host /home/peter/.pgrx --log-file reviews/task-111e/001-candidate-frontier-harness/artifacts/suite-dry-run.log
dry-run succeeded
```

## Review Ask

Please review whether the added sidecar frontier diagnostics and the initial
50k dense RaBitQ-1 f32-oracle suite are the right first Task 111e measurement
surface before we spend time on the full candidate-containment run.
