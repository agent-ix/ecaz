# Task 130: Prune failed TQ experiments; land only validated TQ work to main

Status: **proposed** (2026-06-30). Owner: coder (to be assigned). Priority: P1.

## Why

Task 124 produced a mix of validated wins and failed experiments. Per user
directive, the validated work should be saved and the failed work pruned —
**failed things must not land in `main`.** This task curates the
`task-124-ivf-tq-stage2` branch into a clean, mergeable set.

## KEEP — validated, eligible for main

- The 4-bit no-QJL `turboquant` index-side stage-2 pipeline (disabled-by-default)
  and its attribution counters (pkts 001, 002).
- The validated peripheral scorer wins that produced the only real in-engine
  gain (`-5.4%` scorer-elapsed at 100k, 026 -> 035):
  - query-prep LUT16 specialization (`build_prepared_query_lut_16`, pkt 029);
  - no-QJL batch direct-payload cascade (`candidate_batch`, pkt 030);
  - QJL batch direct-payload cascade (pkt 031);
  - selected-payload loader (pkt 003) + contiguous slab (pkt 011).
- The TQ-internal profiler harnesses (the `#[ignore]` bench tests reporting
  `ns/candidate` and query-prep timing) — needed as measurement infra for
  Tasks 125-129.
- The Phase-6 CLI correctness fix: `ecaz dev evict-relation-cache` now fails
  honestly on macOS instead of falsely reporting `F_NOCACHE` eviction (pkt 019).

## PRUNE — failed / recall-broken, must NOT land in main

All three are validated recall-broken in real indexes (pkts 008/036/037) and are
dead surface:

- `turboquant_binary` — `RerankFormat::TurboQuantBinary = 7` (pkt 007): enum
  variant, reloption parse/name, encode/decode, dispatch, validation, tests.
- `turboquant2` — `RerankFormat::TurboQuant2 = 8` (pkts 008/032): enum variant,
  the `src/quant/qjl2_32/` SIMD kernel module, TQ2 encode in
  `src/am/ec_ivf/quantizer.rs`, dispatch in `candidate_batch`/`scan`/`rerank`,
  options validation, tests.
- `turboquant2_768` — `RerankFormat::TurboQuant2Dim768 = 9` (pkt 037):
  reduced-dimension prefix-subspace path, enum variant, parse/encode/dispatch,
  tests.
- Confirm no `turboquant3` remnant remains (it was reverted; verify).

The failed experiments stay recoverable in branch history and in the packet
`discarded-*.diff` artifacts; they do not need to live in `main`.

## DECISION (owner to confirm, not auto-pruned)

These are stage-2 pipeline controls, not failed formats — keep if the 4-bit
stage-2 pipeline lands:

- `stage2_final_rerank_width`, `rerank_group_width` (the pipeline's own knobs).
- `ec_ivf.tq_stage2_nprobe_cap` (pkt 024) — an opt-in operating-point knob, not a
  TQ speedup; keep as disabled-by-default infra or prune. Owner's call.

## Method

- Land a clean branch off `main` containing ONLY the KEEP set (curate/cherry-pick),
  or revert the PRUNE commits on the branch and verify, then open the PR off that.
- Do NOT merge the `task-124-ivf-tq-stage2` branch wholesale.

## Verification gates

- `rerank_format` parse no longer accepts `turboquant2`, `turboquant2_768`, or
  `turboquant_binary`; `RerankFormat` enum has no `TurboQuant2*`/`TurboQuantBinary`
  variant; `src/quant/qjl2_32/` is gone.
- `cargo build --release -p ecaz`; focused `cargo test` on
  `am::ec_ivf::{options,scan,rerank}`, `am::common::candidate_batch`, and `quant`;
  `cargo clippy --no-default-features --features pg18 -- -D warnings`.
- A recall smoke on the kept 4-bit `turboquant` stage-2 path showing no regression
  vs the pre-prune number.

## Out of scope

- Re-running or re-attempting the failed experiments.
- f32/storage/promotion verdicts.
