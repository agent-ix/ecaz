# Test Quality Strategy — Exhaustive Coverage Plan

**Status:** initial draft (2026-05-18). Owns the "how do we get from
0.00% to 80% on critical modules" question that Task 39's first two
packets leave open. Companion to `plan/tasks/39-test-quality-measurement.md`.

## Why this doc exists

Task 39's first two packets shipped scaffolding:
- `001-test-quality-lanes` — `make coverage` / `make mutants` /
  `make flake-hunt` entrypoints
- `002-coverage-baseline` — `scripts/check_coverage_delta.sh`,
  TSV baseline at **0.00% across every critical module**, GHA jobs

The lanes work. The gate exists. But **the gate is a no-op** because
the baseline is zero, and **coverage is zero** because the pure-Rust
test surface doesn't exercise critical modules. Task 39's task doc
names ≥80% as the production-module target — there's a substantial
gap between current state (0.00%) and exit criterion (≥80%).

This doc inventories that gap and proposes the sequencing to close it.

## Coverage state vs. target

| Module family | Current baseline | Target | Gap |
|---|---|---|---|
| `src/quant/*` (9 modules) | 0.00% | ≥80% | ~80pp |
| `src/storage/page.rs` | 0.00% | ≥80% | ~80pp |
| `src/storage/*_guard.rs` (Task 41 RAII) | not in baseline | ≥80% | full |
| `src/am/*/page.rs` (4 AMs) | 0.00% | ≥80% | ~80pp |
| `src/am/ec_spire/storage/*` | 0.00% | ≥80% | ~80pp |
| `src/am/ec_spire/coordinator.rs` | 0.00% | ≥80% | ~80pp |
| `src/am/ec_diskann/{routine,scan,build}.rs` | 0.00% | ≥80% | ~80pp |
| `src/am/common/cost.rs` | 0.00% | ≥80% | ~80pp |

**Every critical-module target is at 0.00%.** Closing each gap
requires either (a) writing pure-Rust unit tests, or (b) plumbing
pgrx-instrumented coverage. Both are real work.

## Module categorization

Each critical module falls into one of three coverage strategies:

### Category A — pure-Rust coverable (no pgrx required)

These modules have no `pg_sys::*` dependencies in their core logic.
Unit tests in `#[cfg(test)] mod tests` can drive them to high
coverage without a Postgres backend.

- `src/quant/codebook.rs` — k-means / Lloyd-Max codebook computation
- `src/quant/grouped_pq.rs` — grouped-PQ encoding/decoding math
- `src/quant/hadamard.rs` — Hadamard rotation (pure math, SIMD-dispatched)
- `src/quant/mse.rs` — MSE error computation
- `src/quant/prod.rs` — ProdQuantizer encode/decode/score (the
  highest-leverage single file in the codebase)
- `src/quant/qjl.rs` — QJL randomized projection
- `src/quant/rabitq.rs` — RaBitQ quantization
- `src/quant/rotation.rs` — rotation matrix operations
- `src/quant/simd.rs` — SIMD intrinsic wrappers (scalar fallback testable)

**Strategy:** add `#[cfg(test)] mod tests` with table-driven inputs
covering: zero vector, unit vectors, random vectors at d=64/128/256,
boundary cases (NaN, ±inf, denormals), known-answer tests from
adjacent published implementations. Property tests via `proptest`
for round-trip invariants (encode-decode preserves bit pattern up to
documented quantization error).

**Estimated reach per module:** 60-90% line coverage from pure-Rust
tests alone, no pgrx required.

### Category B — pgrx-coupled but testable in `hardening/careful`

These modules use `pg_sys::*` types but can be exercised via the
existing `hardening/careful` lane that uses path-based source imports
without linking the full extension.

- `src/storage/page.rs` — generic page header + tuple layout. The
  `hardening/careful` crate already imports this for the existing
  Miri tests. Coverage measurement just needs the lane to run under
  `cargo-llvm-cov`.
- `src/am/*/page.rs` (per-AM page codecs) — similar; the existing
  `tests/size_of_assertions.rs` (Task 42) exercises some of these
  in pure-Rust mode.

**Strategy:** extend the `careful` harness to:
1. Run all existing tests under coverage instrumentation
2. Add explicit round-trip tests for every page-level encoder/decoder
3. Negative tests: truncated buffers, oversized headers, mismatched
   format versions (Task 42 has fixtures for some of these — reuse them)

**Estimated reach:** 50-80% line coverage per page module from the
`careful` extension.

### Category C — irreducibly pgrx-required

These modules drive actual Postgres state and can only meaningfully
run under `cargo pgrx test pg18`:

- `src/storage/*_guard.rs` (Task 41 RAII) — the Drop semantics
  fire during `pgrx::error!` unwind which requires real pgrx
  panic→elog plumbing
- `src/am/ec_spire/coordinator.rs` — coordinates against real PG
  catalog state, SPI, libpq
- `src/am/ec_diskann/{routine,scan,build}.rs` — exercises only via
  full INSERT / scan / vacuum cycles
- `src/am/ec_spire/storage/*` — same: full AM lifecycle
- `src/am/common/cost.rs` — planner callbacks fire only when PG
  costs an actual query

**Strategy:** pgrx-instrumented coverage. The deferred deliverable
from Task 39's task doc. Steps:
1. Determine whether `cargo pgrx test pg18` can run under
   `cargo-llvm-cov` (requires investigation — pgrx's build flow
   may or may not propagate `RUSTFLAGS="-C instrument-coverage"`).
2. If yes, capture coverage from the existing `make pg-test` runs.
3. If no, alternative: introduce `#[cfg(feature = "pg-test-coverage")]`
   shim that runs a curated subset of `pg-test` cases inside a
   `pgrx::Spi::connect`-style harness with coverage attached.

**Estimated reach:** 70-90% if pgrx-instrumented coverage works
end-to-end; 40-60% if we have to pick-and-choose subsets via shim.

## Mutation testing strategy

Coverage alone is necessary but insufficient. A test that calls
`prod::encode()` but doesn't `assert_eq!` on the output still
contributes to coverage but kills no mutants.

### Mutation-target priority list (top-down)

1. **`src/quant/prod.rs`** — highest single-module leverage. Bugs
   here silently corrupt every AM. Mutation testing here is the
   tightest feedback loop on quantizer correctness.
2. **`src/quant/grouped_pq.rs`** — recently rewritten for PqFastScan;
   high mutation surface.
3. **`src/storage/page.rs`** — page header bit-flips and offset
   mutations are exactly the bugs that corrupt on-disk format.
4. **`src/am/*/page.rs`** (per AM, in order: ec_hnsw, ec_diskann,
   ec_ivf, ec_spire) — tuple kind discriminants and offset arithmetic.
5. **`src/storage/*_guard.rs`** — Drop-impl mutations (replace
   `UnlockReleaseBuffer` with `ReleaseBuffer`) should be caught by
   any test that exercises the error path. Currently no such test
   exists — see Category C above.
6. **`src/am/common/cost.rs`** — cost model arithmetic mutations
   that would silently flip the planner's choice between AMs.

### Mutation-survival triage rules

Per the task doc, every surviving mutant must be triaged as:
1. **"Add a test"** — most common; the test suite missed an
   assertion. Land the test before next packet ships.
2. **"Equivalent mutant"** — mathematically indistinguishable
   from the original (e.g., `<` vs `<=` when the test inputs
   never produce equality). Document why and `--exclude` it.
3. **"Bug filed"** — the mutant exposes a real bug, surface
   into a bugfix packet.

No "ignored" status. Every survivor needs a verdict.

## Flake hunting strategy

Stochastic test lanes (proptest, fuzz, sanitizer) can silently
narrow over time as their seed selection biases toward easy
inputs. The flake-hunt lane re-runs each lane under N alternate
seeds and reports:
- New failures (regression)
- New path coverage in fuzz corpora (signal that the lane is
  finding new things — good)
- Identical paths across all seeds (signal that randomization
  has degraded to deterministic — bad, file a follow-up)

### Lane inventory

| Lane | Seed surface | Re-seed strategy |
|---|---|---|
| `proptest_quant` | `proptest::TestRunner::config().with_seed(...)` | 8 random u64 seeds per nightly run |
| `proptest_page` | same | same |
| `fuzz_parse_text` (cargo-fuzz) | libFuzzer corpus | 8 libfuzzer runs with `-seed=N` |
| `fuzz_unpack_mse` | same | same |
| `fuzz_element_tuple_decode` | same | same |
| `fuzz_neighbor_tuple_decode` | same | same |
| (Task 38 fault injection, when landed) | scenario seeds | 8 alternate scenarios |
| (Task 37 crash recovery, when landed) | SIGKILL timing | 8 alternate WAL boundaries |

Failures from any seed file as bug packets immediately. Nightly
flake-hunt does NOT block PRs.

## Sequencing — proposed next 6 Task 39 packets

| Packet | Surface | Outcome |
|---|---|---|
| **003** Quant pure-Rust unit tests | Category A, top-leverage modules (`prod.rs`, `grouped_pq.rs`, `hadamard.rs`, `codebook.rs`) | Move 4 baselines from 0.00% to ≥60%. First non-trivial coverage delta-gate evidence. |
| **004** Add `src/storage/*_guard.rs` to baseline TSV | Inventory the Task 41 RAII surface, add at 0.00% with explicit note. Add ratcheting policy doc. | Closes the audit gap. |
| **005** Page codec pure-Rust tests | Category B, via `hardening/careful` extension | `src/storage/page.rs` + `src/am/*/page.rs` → ≥50% |
| **006** pgrx coverage feasibility probe | Investigate whether `cargo pgrx test pg18 + cargo-llvm-cov` works end-to-end | Decision: full pgrx-instrumented coverage vs. shim-based subset |
| **007** First mutation triage packet | Run `cargo-mutants --file src/quant/prod.rs`, triage every survivor per the rules above | First validated mutation evidence; raise `prod.rs` mutation-survival bar |
| **008** Coverage ratchet mechanism | Decide manual-update vs. auto-ratchet, document, wire into CI | Removes the "delta gate is a no-op" failure mode |

After packet 008, the gate is genuinely enforcing forward progress
on critical-module coverage, and the mutation lane has at least one
module with real survival data.

## Exit criteria check vs. task doc

Task 39's task doc names:
- `make coverage` runs in CI per-PR with a delta gate — **partially done**
  (lane exists, gate exists, gate is no-op until baselines are non-zero)
- `make mutants` per critical module runs weekly with green or
  triaged-with-followups status — **not started**
- `make flake-hunt` runs nightly — **partially done** (lane exists,
  no re-seed strategy committed yet)
- `docs/hardening.md` gains a "test quality" section — **done**

The first criterion is the long pole. Closing it requires the
sequencing above to land at least packets 003 + 005 (Category A + B
coverage) plus 008 (ratchet mechanism). pgrx coverage (006/Category C)
remains the heaviest lift and may run on a separate schedule from the
80% target.

## Open questions

1. **Pgrx-instrumented coverage feasibility.** Until 006 lands,
   we don't know if Category C is reachable. Worst case: ~30% of
   the critical-module surface stays uncovered indefinitely.
2. **Coverage delta tolerance.** Current ≤2pp drop is fine
   per-file but doesn't enforce upward ratchet. Options in #008
   above.
3. **Mutation cost.** First `--file src/quant/prod.rs` run on
   modern hardware: ~30 min. Full `--workspace` on the target
   list: likely several hours weekly. Confirm the weekly slot is
   resourced.
4. **Coordination with Task 41 RAII guards.** Drop-semantics
   mutation testing (`UnlockReleaseBuffer` → `ReleaseBuffer`)
   only fires meaningfully if a test exercises the error-path
   unwind. That's Category C territory — depends on pgrx
   coverage to be useful.
5. **Coordination with Task 36 (SIMD differential).** Task 36's
   scalar-vs-SIMD proptest harness IS coverage for `src/quant/simd.rs`
   when it lands. Should the Task 39 baseline TSV account for
   Task 36 contributing coverage to that module? Probably yes,
   via a note in the baseline.

## Maintenance

This doc is the source of truth for **why** each Task 39 packet
ships. Update it when:
- A packet lands that moves a baseline value
- The pgrx coverage feasibility question gets answered
- The mutation triage produces survival-rate data
- The flake-hunt seed strategy stabilizes
