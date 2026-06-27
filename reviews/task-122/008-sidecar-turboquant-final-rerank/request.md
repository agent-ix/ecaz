# Task 122 Packet 008: TurboQuant Sidecar Final-Rerank Harness

This packet adds the measurement surface needed to compare TQ as a candidate
reducer before exact f32 rerank. It is a CLI benchmark harness checkpoint only;
no PostgreSQL index behavior is changed.

## Scope

- Adds a `turboquant4` variant to `ecaz bench sidecar-rerank`.
- Adds optional `--final-rerank-k`, which takes the sidecar top-M candidates and
  reranks only that prefix with exact f32 scoring before computing recall.
- Threads `final_rerank_k` through `ecaz bench suite` sidecar-rerank steps.
- Extends sidecar output with `final_rerank_k` and final exact rerank timing
  columns so total bound latency includes candidate SQL, sidecar work, and the
  exact f32 top-M pass.

## Evidence

- Manifest: `artifacts/manifest.md`
- Test log: `artifacts/cargo-test-ecaz-cli-sidecar-rerank.log`

Validation command:

```sh
cargo test -p ecaz-cli sidecar_rerank > reviews/task-122/008-sidecar-turboquant-final-rerank/artifacts/cargo-test-ecaz-cli-sidecar-rerank.log 2>&1
```

Result:

```text
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 405 filtered out; finished in 0.00s
```

## Task 122 Fit

The task definition has seven exploration areas for optimizing TQ. This packet
targets:

- Phase 3: TQ as candidate reducer before f32 rerank.
- Phase 7: correct comparator matrix.

Packets 006 and 007 showed no meaningful SPIRE TQ latency/storage advantage
over RaBitQ in the width-25 matrix. This packet moves the next comparator to a
different shape: fetch a RaBitQ/IVF `rerank=off` frontier, rerank that frontier
with a local TQ sidecar, then optionally exact-rerank only the sidecar top-M.

## Review Notes

The new path is intentionally local to `ecaz-cli`:

- `turboquant4` uses `ProdQuantizer::cached(dim, 4, seed)` and stores encoded
  sidecar payloads beside the existing f32/f16/RaBitQ sidecars.
- DB read modes can persist/fetch TQ sidecar bytes through the existing sidecar
  table path.
- `--final-rerank-k` is validated as `k <= final_rerank_k <= candidate_k`.
- When final rerank is off, sidecar order is preserved and final rerank timing
  reports zero.

This is not a Task 122 closeout request. The next packet should run the actual
10k/50k/100k sidecar suite with recall, latency, and sidecar/storage bytes for
the relevant variants.
