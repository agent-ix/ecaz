# Review request: quant/careful coverage baseline raise

## Scope

This packet addresses the first Task 39 feedback gap from `003-exhaustive-test-coverage-plan`: make the coverage gate measure real critical code instead of only preserving `0.00%` baselines.

Code checkpoint: `0bfd30786b5636258eb8f988185eb5faf1397ca1`

Changes under review:

- Added the real `src/quant` module to the `hardening/careful` harness so `make coverage` now includes quant tests and `src/storage/page.rs` coverage.
- Split coverage artifacts into root and careful summaries, then merged `summary.txt` by canonical path using `scripts/merge_coverage_summaries.py`.
- Added `scripts/check_coverage_baseline_complete.sh` and `make coverage-baseline-check` so critical Task 39 paths cannot silently disappear from `fixtures/quality/coverage-baseline.tsv`.
- Added explicit `--ratchet` mode to `scripts/check_coverage_delta.sh` for reviewed baseline raises.
- Raised quant and `storage/page.rs` baselines in `fixtures/quality/coverage-baseline.tsv`; added `src/storage/*_guard.rs` baseline entries.
- Updated `docs/hardening.md` with the current coverage/mutation/flake gate interpretation and the still-open gaps.

## Key results

From `artifacts/coverage/summary.txt`:

- `quant/codebook.rs`: `98.15%` line coverage
- `quant/grouped_pq.rs`: `94.72%`
- `quant/hadamard.rs`: `92.70%`
- `quant/mse.rs`: `100.00%`
- `quant/prod.rs`: `93.02%`
- `quant/qjl.rs`: `100.00%`
- `quant/rabitq.rs`: `81.43%`
- `quant/rotation.rs`: `98.53%`
- `quant/simd.rs`: `48.00%`
- `storage/page.rs`: `76.57%`
- merged total: `4.51%`

Validation artifacts:

- `artifacts/careful-lib-tests.log`: `90 passed; 0 failed`
- `artifacts/coverage-baseline-check.log`: `coverage baseline complete for 40 critical paths`
- `artifacts/coverage-delta-check.log`: all 40 baseline paths pass against the raised TSV
- `artifacts/coverage-ratchet.log`: records the explicit baseline ratchet command/output
- `artifacts/make-n-task39-quality.log`: dry-run command surfaces for coverage, baseline check, mutation, full mutation, and flake hunt

## Known gaps

This is not a Task 39 closeout. The following feedback items remain open:

- `quant/simd.rs` is measured but only at `48.00%`, below the target.
- `quant/mod.rs` remains `0.00%`.
- AM page callbacks, SPIRE storage/coordinator paths, DiskANN build/routine/scan, planner cost callbacks, and storage guard drops remain `0.00%` until pgrx/integration coverage or additional harnessing is added.
- Mutation testing still needs a real survivor triage packet with verdicts.
- The flake-hunt lane still needs burn-in evidence before it can be promoted beyond candidate status.
