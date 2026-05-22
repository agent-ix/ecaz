# Current-Head PG18 RaBitQ / IVF / SPIRE Sweep Prep

## Scope

This packet records the merge-validation cleanup and bench handoff state after
the upstream merge and the SPIRE wrapper checkpoints.

Code cleanup committed in `e21d0dd42`:

- Mechanical `cargo fmt --all` cleanup in `crates/ecaz-cli`,
  `hardening/careful`, and `src/quant/simd.rs`.
- Test-only SPIRE DML re-export cfg cleanup in `src/am/mod.rs` and
  `src/am/ec_spire/mod.rs`, so normal PG18 bench builds no longer emit the
  unused-import warning.

## Validation Summary

- `cargo fmt --all -- --check`: passed after cleanup.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed cleanly after cleanup.
- `cargo test --no-run --all-targets --no-default-features --features pg18,bench`:
  passed after cleanup.
- `cargo build -p ecaz-cli --bin ecaz`: passed; current CLI is built.
- `cargo clippy --all-targets --no-default-features --features pg18,bench -- -D warnings`:
  failed on broad existing lint backlog; captured as evidence, not cleared in
  this merge prep slice.

## Bench Handoff

Benchmarks **have now been run** locally for IVF/RaBitQ + SPIRE/RaBitQ on the
10k corpus (SPIRE capped at ≤25k per project rule). Full per-step results
are in `artifacts/results.jsonl` and the formatted per-step logs.

### Headline result vs `benchmarks/task-50-local-baseline/` (head `cc06046`)

- **Recall: unchanged within CI95.** IVF/RaBitQ @ nprobe=8/16 = 0.972 / 0.978
  (baseline 0.974 / 0.978); SPIRE/RaBitQ = 0.988 / 0.996 (baseline
  0.992 / 0.999). No quality regression detected from the task-50
  unsafe-block consolidation.
- **Latency: 5–10× regression at p50/mean across both lanes.** IVF/RaBitQ
  p50 5 ms → 49 ms at nprobe=8; SPIRE/RaBitQ p50 39 ms → 226 ms at nprobe=8.

### Most likely cause — not a source regression

The installed PG18 extension at
`/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
is a **debug build** (248 MB, byte-exact match with `target/debug/libecaz.so`)
left over from today's merge-validation `cargo check`/`test` cycle. The
release build (`target/release/libecaz.so`, 17 MB, mtime 2026-05-20) is from
the baseline day. Debug-mode RaBitQ scoring lacks SIMD inlining and is
expected to be roughly an order of magnitude slower — which matches the
observed shift almost exactly.

Action before any optimization work in AWS: rebuild the extension via the
Makefile release recipe (`cargo pgrx install --sudo --release`) and re-run
this suite. Numbers should snap back to baseline territory.

Full report, evidence, and the prioritized next-step list are in
`artifacts/bench-comparison-report.md`.

## Artifacts

See `artifacts/manifest.md` (now updated with the bench surface and
re-run command) and `artifacts/bench-comparison-report.md`.
