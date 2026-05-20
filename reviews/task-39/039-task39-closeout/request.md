# Task 39 / 039 — Task 39 Closeout

## Goal

Walk every Task 39 exit criterion against `plan/tasks/39-*.md`, cite
the artifacts that satisfy each, and call out what's required for full
closure vs. what is in a "recorded as baseline" state per the task
spec.

## Exit Criteria Audit

`plan/tasks/39-test-quality-measurement.md` lists five exit criteria.
Each is evaluated below against the live repo at this packet's head.

### 1. `make coverage` runs in CI per-PR with a delta gate

**Met.**

- `.github/workflows/ci.yml::test-quality-coverage` runs `make
  coverage` on every push/PR, then `make coverage-baseline-check`,
  then `scripts/check_coverage_delta.sh` against
  `fixtures/quality/coverage-baseline.tsv` with the PR's changed-files
  list (or the full baseline when not a PR).
- `make test-quality-ci-audit` confirms the wiring is intact (see
  `artifacts/test-quality-ci-audit.log`).
- The local coverage lane runs against `ecaz-cli` and
  `hardening/careful` (the supported subset until live pgrx coverage
  is unblocked; see "PG18 instrumentation" gap below).

### 2. `make mutants` per critical module runs weekly with green or triaged status

**Met.**

- `.github/workflows/ci.yml::test-quality-mutants` runs
  `make mutants-full` on a weekly schedule
  (`37 10 * * 1`) and on `workflow_dispatch`, with artifact upload.
- Most recent baseline: `reviews/task-39/027-rabitq-mutation-closeout/`
  shows the full RaBitQ sweep ending at **447 mutants, 0 missed, 0
  timed-out**.

### 3. `make flake-hunt` runs nightly

**Met.**

- `.github/workflows/ci.yml::test-quality-flake-hunt` runs
  `make flake-hunt FLAKE_HUNT_SEEDS=8 FLAKE_HUNT_FUZZ_SECONDS=10`
  nightly (`37 9 * * *`) and on `workflow_dispatch`, with artifact
  upload.

### 4. `docs/hardening.md` gains a "test quality" section

**Met.**

- `docs/hardening.md::## Test Quality` (lines 182-299) covers lane
  inventory, gate level table, coverage baseline policy (intentionally
  not duplicating the TSV per the documented "policy not log" split),
  ratchet workflow, mutation triage shape, cross-arch mutation
  pattern, and flake-hunt seed budgeting.

### 5. Baseline coverage % per critical module is recorded

**Met.**

- `fixtures/quality/coverage-baseline.tsv` records baselines for
  **40 critical paths** validated by
  `scripts/check_coverage_baseline_complete.sh`. The set covers
  `src/quant/**`, `src/storage/page.rs` + per-AM `page.rs`, every
  `src/am/ec_spire/storage/*.rs`, `src/am/ec_spire/coordinator/diagnostics.rs`,
  the three named DiskANN core modules, and `src/am/common/cost.rs`.

## Current Baseline State

Above the documented 80% burn-in target:

| File | Baseline | Last raise |
| --- | ---: | --- |
| `am/ec_spire/page.rs` | 83.15 | 037-relation-store-chain |
| `am/ec_spire/storage/leaf_v2_parts.rs` | 81.03 | 032 |
| `am/ec_spire/storage/local_store.rs` | 80.07 | 032 |
| `am/ec_spire/storage/leaf_v2.rs` | **95.29** | 036-coverage-pushes |
| `am/ec_spire/storage/vec_id.rs` | **94.64** | 036-coverage-pushes |
| `am/ec_spire/storage/local_store_set.rs` | **88.89** | 036-coverage-pushes |
| `quant/rabitq.rs` | 81.43 | (legacy) |
| 24 other critical paths | 82.00 — 100.00 | various |

Below 80% but recorded:

| File | Baseline | Reason |
| --- | ---: | --- |
| `am/ec_spire/storage/relation_store.rs` | 58.10 | Phase-1 emulator covers single + multi-tuple round trips, chain codecs, trait dispatch, prefetch grouping; the remaining ~22% is dominated by crafted-byte error branches and the PG18-only read-stream loop (emulator stub). See packet 038 for the explicit follow-up list. |
| `am/ec_spire/coordinator/diagnostics.rs` | 0.00 | File is `include!`'d into `ec_spire/mod.rs` and pulls coordinator/types.rs (1833 lines of snapshot row structs), `quantizer::*`, an Spi surface, and four accumulator types from sibling coordinator files. Careful-side scaffolding is a multi-packet effort tracked as Task 39 follow-up; live pgrx coverage is the other path once the PG18 instrumentation gap closes. |
| `am/ec_diskann/routine.rs` | 0.00 | 4291 lines using `pgrx::PgBox`, `pgrx::FromDatum`, `pgrx::PgMemoryContexts`, and `extern "C-unwind"` callbacks. Scaffolding requires substantial new pgrx-bindings shadow types and is the largest individual coverage gap; same blocker on live pgrx for the alternative path. |

The Task 39 spec explicitly frames the baseline as "recorded so future
regressions are visible" — not as a hard floor. The 80% target is a
"after burn-in" guidance in `docs/hardening.md::## Test Quality`, not
an exit criterion. The two 0.00% baselines pin the current state so
any regression is immediately visible to CI.

## Known Open Gap: Live pgrx Coverage on PG18

`docs/hardening.md::## Test Quality` documents the current PG18
instrumentation block: `RUSTFLAGS="-C instrument-coverage"` builds
the pgrx test profile but the lib test binary aborts before execution
on macOS PG18 (`dyld` fails to resolve `_BufferBlocks`), and the
profile-writer needs an absolute `LLVM_PROFILE_FILE`. Until those are
fixed, the supported coverage surface is the shim-based subset
exercised by `make coverage`: `ecaz-cli` plus `hardening/careful`.

This is the upstream blocker for raising `diagnostics.rs` and
`routine.rs` without writing new careful-side scaffolds.

## This Session's Coverage Wins

Packets `034`-`038` (plus `036`-`038`'s ratchets) moved:

| File | Pre-session | Now |
| --- | ---: | ---: |
| `am/ec_spire/page.rs` | 0.00 | **83.15** |
| `am/ec_spire/storage/relation_store.rs` | 0.00 | **58.10** |
| `am/ec_spire/storage/leaf_v2.rs` | 71.76 | **95.29** |
| `am/ec_spire/storage/local_store_set.rs` | 41.52 → 63.74 | **88.89** |
| `am/ec_spire/storage/vec_id.rs` | 69.05 → 69.64 | **94.64** |
| `am/ec_spire/storage/leaf_v2_parts.rs` | 77.52 → 81.03 | 81.03 |
| `am/ec_spire/storage/local_store.rs` | 78.21 → 80.07 | 80.07 |

`am/ec_spire/page.rs` and `am/ec_spire/storage/relation_store.rs` were
both at 0% at session start; both now have real round-trip coverage
through the Phase-1 backing-page emulator (packet 035) plus chain and
trait-dispatch coverage (packets 037-038). The careful test suite
went from 455 → **500 passing** across the session.

## Validation

Artifacts under `reviews/task-39/039-task39-closeout/artifacts/`:

- `closeout-focused-tests.log`: `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib` → **500 passed, 0 failed**.
- `coverage/summary.txt` + JSON files: full `make coverage` output at
  closeout head.
- `coverage-delta-check.log`: full-baseline delta check (every
  recorded path green at its baseline).
- `coverage-baseline-check.log`:
  **coverage baseline complete for 40 critical paths**.
- `test-quality-ci-audit.log`: `Task 39 CI audit passed` for all
  three Task 39 CI lanes.

## Reviewer Direction

All five Task 39 exit criteria are met. Two non-criterion gaps remain
visible against the documented 80% burn-in target
(`relation_store.rs`, `diagnostics.rs`, `routine.rs`); each is
captured above with a specific blocker (crafted-byte error tests for
relation_store, deep coordinator/pgrx scaffolds for the other two).
The pgrx instrumentation gap on macOS PG18 is the upstream blocker
for the 0% files via the live-backend route.

If reviewer agrees, Task 39 can close on the criteria with the three
gaps tracked as named follow-ups under the same task bucket; the
recorded baselines guarantee no silent regression in the meantime.
