# Task 39: Test Quality Measurement (Mutation, Coverage, Flake Hunting)

Status: **proposed** — answers the question Task 34 cannot: "do the tests we
already have actually catch bugs, and what do they leave uncovered?"

## Scope

Add two measurement lanes to the hardening stack:

1. **Coverage** via `cargo-llvm-cov` (line + region) over `make hardening-local`,
   `make pg-test`, and `make hardening-nightly-local`. Report stored as a
   packet artifact; CI delta-reports per PR.
2. **Mutation testing** via `cargo-mutants` over critical correctness modules,
   run on a slower cadence (weekly or per-release).
3. **Flake hunting**: re-run randomized lanes (proptest, fuzz, sanitizer, crash
   recovery) under different seeds and report any non-determinism.

Coverage targets (mutation):

- `src/quant/**` — every quantizer encode/decode/score path.
- `src/storage/page.rs` and `src/am/*/page.rs` — page codec / tuple layout.
- `src/am/ec_spire/storage/**`, `src/am/ec_spire/coordinator.rs` — SPIRE state
  machine.
- `src/am/ec_diskann/{routine,scan,build}.rs` — DiskANN core paths.
- `src/am/common/cost.rs` — planner cost model.

## Why

Coverage and mutation testing answer different questions, and ECAZ has neither:

- **Coverage** tells you which lines/regions a test suite reaches. Without it
  we cannot tell whether new Miri/fuzz/Kani lanes from Task 34 actually
  exercise SPIRE coordinator code (they do not — Task 34 review confirmed),
  or whether the unit tests skip entire error paths.
- **Mutation testing** tells you whether your tests *assert* on what they
  execute. Coverage of a function that has only `assert!(do_thing())` (and
  not the result it produces) is misleading. Mutation testing replaces
  operators (`+` → `-`, `<` → `<=`, `true` → `false`) and reports surviving
  mutants — places where a wrong implementation passes the tests.
- **Flake hunting** matters because randomized lanes (proptest, fuzz,
  sanitizer with stochastic schedules) can silently regress to deterministic
  inputs that no longer find anything. Re-seeding is cheap insurance.

Task 34 shipped ~10 lanes "passing" but ran zero of them with coverage. The
review found that four of them ran against synthetic shadow crates — coverage
would have surfaced this immediately. Mutation testing on top would have
revealed how much of the *real* coverage is asserted vs. merely executed.

## Approach

1. **Coverage lane.**
   - `cargo install cargo-llvm-cov` checked by `scripts/hardening.sh`.
   - Aggregate report under `review/<packet>/artifacts/coverage/` with HTML +
     summary CSV (per-file, per-region).
   - Two thresholds in the report: "production modules" (≥ 80% target after
     burn-in) and "test/bench/helpers" (no threshold).
   - PR delta: a CI job that diffs coverage % per file vs. base branch and
     posts a comment. Drop > 2 percentage points on a touched file requires
     review-packet justification.
2. **Mutation lane.**
   - `cargo install cargo-mutants`.
   - Per-module `--file` invocations so a single mutation run targets a
     bounded blast radius (full repo is hours).
   - Initial target list: SIMD scoring, page codecs, SPIRE coordinator,
     planner cost model.
   - Surviving mutants land in `review/<packet>/artifacts/mutants.json` and
     `mutants.txt`; the packet must triage each one as either:
     - "add a test that kills it,"
     - "intentional (equivalent mutant)," or
     - "filed as follow-up bug."
3. **Flake hunting.**
   - Proptest, fuzz, and sanitizer lanes accept a `--seed` argument or env.
   - Nightly lane re-runs each with N different seeds (default 8) and reports
     any failure or new path discovery.
   - Crash recovery (Task 37) accepts a `--crash-seed` and is included.
4. **Make lanes:**
   - `make coverage` — collects coverage, writes to `target/llvm-cov/`.
   - `make coverage-report` — uploads/copies to packet artifacts.
   - `make mutants MODULE=src/quant/prod.rs` — per-module mutation run.
   - `make mutants-full` — weekly full sweep across the target list.
   - `make flake-hunt` — re-seed sweep over randomized lanes.

## Validation

- Coverage report includes data from every `hardening-local` and `pg-test`
  lane (including pgrx integration tests where instrumentation is feasible).
- Mutation lane produces a non-empty mutants list on first run; surviving
  mutants are triaged in the inaugural packet (expected: many).
- A deliberately weakened assertion (`assert_eq!(x, y)` → `assert!(true)`)
  produces a surviving mutant that the lane flags.
- Flake hunt over fuzz at 8 seeds finds no new crashes; runs complete within
  the seed budget.

## Exit Criteria

- `make coverage` runs in CI per-PR with a delta gate.
- `make mutants` per critical module runs weekly with a green or
  triaged-with-followups status.
- `make flake-hunt` runs nightly.
- `docs/hardening.md` gains a "test quality" section explaining the
  measurement model, thresholds, and how to interpret mutation results.
- A baseline coverage % per critical module is recorded in `docs/hardening.md`
  so future regressions are visible.

## Dependencies

- Independent of other proposed hardening tasks.
- Coverage data is the most useful prerequisite for Task 49 (CI promotion) —
  promotion decisions should cite coverage of the lane being promoted.
- Mutation results inform Task 36 (SIMD diff) and Task 47 (recall gates) by
  showing where existing tests are weak.
