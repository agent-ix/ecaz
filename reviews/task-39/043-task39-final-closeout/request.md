# Task 39 / 043 — Final Closeout

Supersedes the partial closeout in packet 039 with the helpers-extract
and additional ratchets shipped in 040, 041, and 042.

## Exit Criteria Audit

All five criteria in `plan/tasks/39-test-quality-measurement.md` are
met. Evidence per criterion:

| Criterion | Status | Evidence |
| --- | --- | --- |
| `make coverage` runs in CI per-PR with a delta gate | **Met** | `.github/workflows/ci.yml::test-quality-coverage`; `make test-quality-ci-audit` confirms (`artifacts/test-quality-ci-audit.log`); the lane runs against `ecaz-cli` plus `hardening/careful`. |
| `make mutants` per critical module runs weekly with green/triaged status | **Met** | `.github/workflows/ci.yml::test-quality-mutants` on weekly cron `37 10 * * 1`; most-recent full sweep `reviews/task-39/027-rabitq-mutation-closeout/` shows **447 mutants, 0 missed**. |
| `make flake-hunt` runs nightly | **Met** | `.github/workflows/ci.yml::test-quality-flake-hunt` on nightly cron `37 9 * * *` with `FLAKE_HUNT_SEEDS=8`. |
| `docs/hardening.md` gains a "test quality" section | **Met** | `docs/hardening.md::## Test Quality` (lines 182-299) covers lanes, gate-level table, ratchet workflow, mutation triage shape, cross-arch mutation pattern, and flake-hunt seed budgeting. |
| Baseline coverage % per critical module is recorded | **Met** | `fixtures/quality/coverage-baseline.tsv` records baselines for **42 critical paths** validated by `scripts/check_coverage_baseline_complete.sh`. |

## Coverage State at Closeout

### Above the documented 80% burn-in target

| File | Baseline | Last raise |
| --- | ---: | --- |
| `am/ec_diskann/routine_helpers.rs` | **100.00** | 041 |
| `am/ec_spire/coordinator/diagnostics_helpers.rs` | **100.00** | 040 |
| `am/ec_spire/storage/leaf_v2.rs` | **95.29** | 036 |
| `am/ec_spire/storage/vec_id.rs` | **94.64** | 036 |
| `am/ec_spire/storage/local_store_set.rs` | **88.89** | 036 |
| `am/ec_spire/page.rs` | 83.15 | 037 |
| `am/ec_spire/storage/leaf_v2_parts.rs` | 81.03 | 032 |
| `am/ec_spire/storage/local_store.rs` | 80.07 | 032 |
| `quant/rabitq.rs` | 81.43 | (legacy) |
| 24 other critical paths | 82.00 — 100.00 | various |

### Below 80% but recorded

| File | Baseline | Reason |
| --- | ---: | --- |
| `am/ec_spire/storage/relation_store.rs` | 58.52 | Backing-page emulator (packet 035) covers single + multi-tuple round trips, chain codecs, trait dispatch, prefetch grouping, placement validation, and length-mismatch reader branches. Remaining ~22% is the crafted-byte chain corruption branches (segment-no, byte-base, trailing locator) and the PG18 read-stream prefetch loop (emulator stub). |
| `am/ec_spire/coordinator/diagnostics.rs` | 0.00 | The 11 pure helpers were extracted to `diagnostics_helpers.rs` (100% covered, packet 040). The remaining body is the pgrx-touching snapshot/accumulator surface (relation reads, manifests, Spi) that needs live PG18 or a deeper coordinator scaffold. |
| `am/ec_diskann/routine.rs` | 0.00 | The 3 pure helpers were extracted to `routine_helpers.rs` (100% covered, packet 041). The remaining 1522 lines are pgrx-FFI callback code (`amhandler`, vacuum callbacks, bulk insert, scan-state construction) that needs live PG18. |

The Task 39 spec frames the baseline as "recorded so future
regressions are visible," not as a hard floor. The 80% target is
"after burn-in" guidance in `docs/hardening.md::## Test Quality`, not
an exit criterion. The three recorded 0.00% / 58.52% baselines pin
their current state so any regression is immediately visible to CI.

## Known Open Gap: Live pgrx Coverage on PG18

`docs/hardening.md` documents the current PG18 instrumentation block:
`RUSTFLAGS="-C instrument-coverage"` builds the pgrx test profile but
the lib test binary aborts before execution on macOS PG18 (`dyld`
fails to resolve `_BufferBlocks`), and the profile-writer needs an
absolute `LLVM_PROFILE_FILE`. Until those are fixed, the supported
coverage surface is the shim-based subset exercised by `make coverage`:
`ecaz-cli` plus `hardening/careful`. This is the upstream blocker for
raising the three 0%/58% baselines without writing larger careful-side
scaffolds for the coordinator and diskann pgrx callback surface.

## This Session's Coverage Wins

Session packets `034`-`042` moved:

| File | Pre-session | Now |
| --- | ---: | ---: |
| `am/ec_spire/page.rs` | 0.00 | **83.15** |
| `am/ec_spire/storage/relation_store.rs` | 0.00 | **58.52** |
| `am/ec_spire/storage/leaf_v2.rs` | 71.76 | **95.29** |
| `am/ec_spire/storage/local_store_set.rs` | 63.74 | **88.89** |
| `am/ec_spire/storage/vec_id.rs` | 69.64 | **94.64** |
| `am/ec_spire/storage/local_store.rs` | 78.21 → 80.07 | 80.07 |
| `am/ec_spire/storage/leaf_v2_parts.rs` | 77.52 → 81.03 | 81.03 |
| `am/ec_spire/coordinator/diagnostics_helpers.rs` | did not exist | **100.00** |
| `am/ec_diskann/routine_helpers.rs` | did not exist | **100.00** |

`am/ec_spire/page.rs` and `am/ec_spire/storage/relation_store.rs` were
both at 0% at session start; both now have real round-trip coverage
through the Phase-1 backing-page emulator (packet 035) plus chain,
multi-segment, trait-dispatch, validate-placement, and codec-error
coverage (packets 037-038, 042).

`coordinator/diagnostics.rs` and `ec_diskann/routine.rs` each gave up
their pure helpers to sibling `*_helpers.rs` files at 100% coverage
(packets 040, 041), shifting ~210 previously-unreachable lines into
the supported coverage surface without changing production behavior.

The careful test suite went from **455 → 513 passing** across the
session.

## Validation

Artifacts under `reviews/task-39/043-task39-final-closeout/artifacts/`:

- `closeout-focused-tests.log`: `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib` → **513 passed, 0 failed**.
- `coverage/summary.txt` + JSON files: full `make coverage` output at
  closeout head.
- `coverage-delta-check.log`: full-baseline delta check (every
  recorded path green at its baseline).
- `coverage-baseline-check.log`:
  **coverage baseline complete for 42 critical paths**.
- `test-quality-ci-audit.log`: `Task 39 CI audit passed` for all
  three Task 39 CI lanes.

## Reviewer Direction

All five Task 39 exit criteria are met. The two remaining 0% and the
one 58.52% baseline are blocked on the documented live-pgrx-coverage
gap; their recorded baselines guarantee no silent regression in the
meantime. If reviewer agrees, Task 39 can close on the criteria with
the three gaps tracked as named follow-ups under this task bucket.

The session also confirmed the "extract pure helpers" pattern as the
cheap escape valve for 0% pgrx-bound files when live-pgrx coverage is
blocked — packets 040 and 041 both apply it without behavior change.
The same pattern is available for any future 0% file where the live
backend is the only path to its core surface.
