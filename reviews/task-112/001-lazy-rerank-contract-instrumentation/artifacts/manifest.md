# Task 112 — Packet 001 Manifest: Lazy Heap-F32 Rerank (Contract + Instrumentation + Gate)

- Head SHA at packet authoring: `61fd84f95` (base) → code commit on branch
  `task-112-ivf-lazy-heap-f32-rerank` (see `request.md` for the exact code SHA).
- Task bucket: `reviews/task-112/`
- Packet path: `reviews/task-112/001-lazy-rerank-contract-instrumentation/`
- Lane / host: dev/review box (Linux WSL2 PG18 pgrx test cluster). This box has
  **no `ecaz` binary and no staged corpora**, so Phase 5 bench evidence is
  deferred to the Intel bench desktop.
- Storage format under test: `coarse_rerank` heap-f32 table-side rerank
  (the only surface Task 112 touches). No on-disk format change.
- Rerank mode: `heap_f32` (`rerank_format = 'f32'`, `rerank_placement = 'table'`).

## Artifacts

| file | what | how produced |
|---|---|---|
| `cargo-check.log` | `cargo check --no-default-features --features pg18` — clean | command below |
| `cargo-clippy.log` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — clean | command below |
| `cargo-test-lazy-unit.log` | lazy module unit tests + the 7 Task-112 pg_test cases (filter `lazy`) — 13 passed | command below |
| `cargo-test-explain-unit.log` | `IvfExplainCounters` unit tests incl. new counters — 9 passed | command below |
| `task-112-lazy-rerank-ab.intel-local.json` | deferred Phase-5 `ecaz bench suite` A/B config (lazy on vs off at matched recall) | hand-authored; JSON validated |

## Commands

```text
cargo check --no-default-features --features pg18
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
cargo test --no-default-features --features pg18 --lib lazy
cargo test --no-default-features --features pg18 --lib 'am::common::explain::tests'
cargo pgrx test pg18 lazy_rerank
```

Timestamp: 2026-06-19 (PDT), on the dev/review box.

## Key result lines

- Lazy unit tests (`cargo-test-lazy-unit.log`):
  `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 2152 filtered out`
  - includes `no_bound_never_stops_early`, `calibrated_bound_can_stop_early`,
    `skipped_candidates_are_provably_not_better_than_kept`, and the 7
    `pg_test_ec_ivf_lazy_rerank_*` cases (equivalence, counters, gate-off, ties,
    duplicate heap TIDs, empty frontier, rerank_width boundaries).
- Explain counter tests (`cargo-test-explain-unit.log`):
  `test result: ok. 9 passed; 0 failed`
  (covers the new `stats_rerank_candidates_considered` /
  `stats_rerank_candidates_skipped` counters + render order).
- `cargo check` / `cargo clippy`: `Finished` with no warnings.

## Pre-existing unrelated failure (not touched)

`pg_test_ec_spire_*` (e.g. `ec_spire.cost_routing_dimension_scale should
increase CustomScan EXPLAIN cost: baseline=2.67, tuned=2.67`) fails on this
checkout independent of IVF work. Task 112 changes nothing under `ec_spire`.

## Phase 5 (deferred — env-blocked)

`task-112-lazy-rerank-ab.intel-local.json` is the ready-to-run A/B. Run on the
Intel bench desktop after staging `ec_real_100k` at `data/staged-current/`:

```text
ecaz bench suite run \
  --config reviews/task-112/001-lazy-rerank-contract-instrumentation/artifacts/task-112-lazy-rerank-ab.intel-local.json \
  --artifact-dir reviews/task-112/001-lazy-rerank-contract-instrumentation/artifacts/bench
```

Evidence Requirements the config emits per arm (lazy off vs on): suite config,
reloptions, query count, recall@10 + NDCG@10, p50/p95/p99 + mean, heap rows
fetched, heap blocks fetched, exact rerank elapsed, approximate scan elapsed,
and skipped-candidate count.

**Expected result under the current contract:** because the sound `NoBound`
default never stops early, lazy-on and lazy-off are byte-identical (skipped == 0)
— equal recall, ordering, heap rows/blocks, and latency within noise. The A/B
confirms recall-neutrality of the gate on a real corpus and becomes the live win
measurement once Task 113 supplies a calibrated lower bound that lets the stop
fire.
