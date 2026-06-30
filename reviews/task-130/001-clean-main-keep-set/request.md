# Review Request: Task 130 Packet 001 - Clean Main-Based TQ Keep-Set

## Summary

This supersedes the earlier incomplete Task 130 branch. The branch is now based
directly on current `origin/main` (`67f23b56b`) and cherry-picks only the
validated Task 124 keep-set.

It does **not** include the recall-broken Task 124 IVF formats:

- no `RerankFormat::TurboQuantBinary`;
- no `RerankFormat::TurboQuant2`;
- no `RerankFormat::TurboQuant2Dim768`;
- no IVF `turboquant_binary`, `turboquant2`, or `turboquant2_768` parse aliases;
- no `src/quant/qjl2_32/`;
- no TQ2 or binary IVF encode/rerank/scan dispatch.

The only `turboquant_binary` references visible on the clean branch are
pre-existing HNSW runtime/test references from `origin/main`, not Task 124 IVF
additions.

## What Landed

Kept:

- 4-bit IVF `rerank_format=turboquant` stage-2 final rerank width.
- TQ stage-2 attribution counters.
- Selected TQ sidecar payload loader and selected-payload slab.
- TQ rerank group-width locality control.
- No-QJL gamma elision.
- LUT16 query-prep improvement.
- Production TQ no-QJL and QJL batch payload cascades.
- TQ LUT32 and prefetch profiler harnesses.
- macOS relation-cache eviction CLI fix.
- `.gitignore` rules for regenerable `truth-*.json` files.

Pruned:

- binary IVF TQ stage-2 format;
- TQ2 IVF stage-2 format;
- reduced-dimension TQ2 IVF format;
- TQ2 SIMD / `qjl2_32`;
- TQ2 dimension/subspace profiler.

## Review Focus

- Confirm the branch diff is cleanly based on `origin/main`, not the full Task
  124 branch.
- Confirm the landing diff contains no IVF enum 7/8/9 recall-broken formats and
  no `qjl2_32`.
- Confirm the retained code is the production 4-bit TQ keep-set described in
  Task 130.

## Validation

Completed on `task-130-tq-cleanup-main`:

- `rg "TurboQuant2|TurboQuantBinary|qjl2_32|turboquant2|turboquant_binary|tq2" src/am/ec_ivf src/am/common/candidate_batch src/quant crates/ecaz-cli`
  - only `rabitq2` false positives in `src/am/ec_ivf/options.rs`;
  - no IVF `TurboQuant2`, `TurboQuantBinary`, `qjl2_32`, `turboquant2`, or
    `turboquant_binary` landing surface.
- `git diff --check` passed.
- `git check-ignore -v reviews/task-130/001-clean-main-keep-set/artifacts/tq4-smoke-suite/truth-10k-k10.json`
  confirms `reviews/**/truth-*.json` ignores generated truth caches.
- `cargo build --release -p ecaz` passed.
- `cargo clippy -p ecaz --lib --no-default-features --features pg18 -- -D warnings`
  passed.
- Focused PG18 Rust tests passed:
  - `am::ec_ivf::options`: 27 passed.
  - `am::ec_ivf::scan`: 30 passed.
  - `am::ec_ivf::rerank`: 20 passed.
  - `am::common::candidate_batch`: 13 passed, 2 explicit profiling tests ignored.
  - `quant`: 305 passed, 2 explicit profiling tests ignored.
- 4-bit `turboquant` recall smoke passed through `ecaz bench suite`:
  - config: `artifacts/task130-tq4-smoke-suite.json`;
  - manifest/results: `artifacts/tq4-smoke-suite/suite-manifest.json`,
    `artifacts/tq4-smoke-suite/results.jsonl`;
  - status: 2 completed, 0 failed, 0 missing artifacts;
  - recall@10: `1.0000` at nprobe 32 and 64 over 100 queries / 1000 trials;
  - mean query time: 0.95 ms at nprobe 32, 1.12 ms at nprobe 64.
