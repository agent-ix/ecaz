# Review Request: Task 130 Packet 001 - Task 124 Cleanup

## Summary

This packet cleans up the two Task 124 issues that should not leak into a landing branch:

- The failed reduced-dimension validation format `rerank_format=turboquant2_768` is removed from the production-facing IVF reloption surface.
- Regenerable `truth-*.json` files emitted by `ecaz bench suite` are now ignored in review/benchmark packet trees.

The packet keeps Task 124's evidence intact. Packet 037 remains the durable proof that reduced-dimension TQ2 produced a real scorer-speed reduction but failed recall at 50k/100k. The cleanup only removes the failed validation-only reloption from callable code.

## Changes

- Reverted code commit `0b3fd57f7` (`Add reduced-dimension TQ2 rerank format`), removing:
  - `RerankFormat::TurboQuant2Dim768`
  - `turboquant2_768` / `turboquant_2_768` / `tq2_768` parsing aliases
  - reduced-dimension prefix-subspace codec plumbing
  - 768D-specific options/rerank tests
- Added `.gitignore` entries:
  - `reviews/**/truth-*.json`
  - `benchmarks/**/truth-*.json`
- Added `plan/tasks/130-tq-post-task124-cleanup.md`.
- Updated Task 124 docs to point at Task 130 for the validation-only reloption cleanup.
- Updated `plan/tasks/README.md` with Task 130.

## Validation

- `rg "TurboQuant2Dim768|turboquant2_768|tq2_768" src/am/ec_ivf`
  - no matches
- `git check-ignore -v reviews/task-124/037-tq2-dim768-real-index/artifacts/tq2-dim768-final15-suite/truth-100k-k10.json`
  - matched `.gitignore:60:reviews/**/truth-*.json`
- `cargo test -p ecaz --lib --no-default-features --features pg18 rerank_format_parse_accepts_turboquant2 -- --nocapture --test-threads=1`
  - passed, 1 test
- `cargo test -p ecaz --lib --no-default-features --features pg18 turboquant2_sidecar_uses_compact_qjl_payload -- --nocapture --test-threads=1`
  - passed, 1 test
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - passed
- `git diff --check`
  - passed

One broad validation attempt, `cargo test ... turboquant`, failed because the filter matched unrelated shared counter tests that ran concurrently and poisoned the shared counter mutex. The focused serial reruns above are the validation basis for this cleanup.

## Review Focus

- Confirm `turboquant2_768` is gone from source while Task 124 packet 037 remains as negative evidence.
- Confirm the `.gitignore` additions match the repo workflow's "never commit regenerable truth caches" rule.
- Confirm the Task 124 / Task 130 documentation keeps production 4-bit TQ speedup separate from smaller recall-broken experimental formats.
