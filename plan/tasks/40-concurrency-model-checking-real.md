# Task 40: Concurrency Model Checking Over Real ECAZ State

Status: **proposed** — replaces the placeholder Loom and Shuttle harnesses
landed in Task 34 with models that exercise the actual ECAZ concurrent code,
and extends concurrency coverage to the SPIRE distributed coordinator using
`madsim` / `turmoil` style deterministic simulation.

## Scope

Three coverage areas:

1. **Shared-memory concurrency (Loom)** — small atomic / lock-free state in
   ECAZ that fits Loom's exhaustive interleaving budget:
   - parallel index-build worker-slot claim/release (`src/am/*/parallel_build*`),
   - DSM handoff for parallel scan (`src/am/common/parallel_scan*`),
   - any `AtomicPtr`/`AtomicUsize`/`Once`-style code in ECAZ glue.
2. **Coarse-grained concurrency (Shuttle)** — larger but still bounded state
   machines:
   - SPIRE coordinator candidate-merge and epoch advance,
   - SPIRE leaf maintenance threshold / promotion logic,
   - DiskANN background prefetch coordination,
   - any future async worker pool.
3. **Distributed simulation (madsim / turmoil)** — only if SPIRE remote
   transport is asynchronous / network-bound:
   - SPIRE remote object fetch with packet loss, reorder, partition,
   - coordinator/replica leader change under message loss,
   - retry / backoff under tail latency.

## Why

Task 34 landed Loom and Shuttle harnesses, but neither imports any ECAZ
type. The "passing" status proves the tools work; it proves nothing about
ECAZ. Concurrent state machines that haven't been model-checked are exactly
where race bugs live. Specific known surfaces:

- Parallel build worker slots use atomic claim/release; an ordering bug
  produces lost rows or double-counting.
- SPIRE coordinator handles epoch transitions across multiple writers and
  scanners — by-construction the place where a model checker pays off.
- Anything tokio-based that crosses the network is non-deterministic in
  tests today; `madsim` lets you reproduce the bad schedule by seed.

The right model is "shadow code that the production code also uses," not "a
re-implementation that may drift." Where the production atomic/lock logic can
be lifted into a pure-Rust helper module, both Loom and the real backend can
depend on it; that is the only way to keep the model honest over time.

## Approach

1. **Lift sync primitives.** For each target surface, extract the concurrent
   protocol into a pure-Rust module (no pgrx) that both prod code and the
   model harness depend on. Examples:
   - `src/am/common/parallel_slot.rs` — a typed wrapper around the atomic
     slot claim/release.
   - `src/am/ec_spire/coordinator/state.rs` — the coordinator's epoch +
     candidate-merge state machine as a pure type, with PG-side glue in a
     separate module.
2. **Loom harness over real types.** Replace `hardening/loom/src/lib.rs`
   shadow code with Loom tests that import the real lifted modules under
   `cfg(loom)` and use `loom::sync` aliases. Add at least:
   - worker-slot exclusive claim under N=2..4 workers,
   - DSM handoff producer/consumer with bounded slots,
   - any reference-counted shared state (Arc / Rc patterns).
3. **Shuttle harness over real types.** Replace
   `hardening/shuttle/src/lib.rs` with Shuttle models that import the SPIRE
   coordinator state machine. Targets:
   - candidate-merge order invariance,
   - epoch monotonicity under concurrent advance,
   - "no scanner observes a partial replacement" — the canonical visibility
     invariant.
4. **`-Zmiri-many-seeds` interleavings.** Use Miri's data-race detector and
   the many-seeds flag to surface schedule-dependent UB in pure-Rust paths
   that are not yet lifted into a model checker. This is cheap incremental
   coverage between dedicated model-check passes.
5. **Madsim / turmoil for SPIRE remote.** If SPIRE remote uses tokio, add
   `crates/ecaz-sim-spire/` that runs the coordinator + remote against
   `madsim`'s deterministic runtime. Tests:
   - reorder / drop / duplicate transport packets,
   - simulated 200ms p99 tail latency,
   - partition heal + leader election (if applicable),
   - clock skew between coordinator and remote.
6. **Make lanes:**
   - `make loom-real` — Loom over lifted ECAZ types.
   - `make shuttle-real` — Shuttle over lifted ECAZ types.
   - `make miri-many-seeds` — Miri with `-Zmiri-many-seeds=128`.
   - `make sim-spire-remote` — madsim/turmoil sweep over SPIRE remote.

## Validation

- Each `make loom-real` test reaches a real lift of production code; running
  `git grep "fn .* loom" hardening/loom/` against `git grep "pub fn" src/`
  shows the dependency.
- Shuttle harness runs the SPIRE coordinator state machine and checks the
  exact same invariants enforced by production assertions.
- Forcing a known race (e.g., release before acknowledgement) produces a
  Loom/Shuttle counterexample reproducible from the printed schedule.
- `make sim-spire-remote` survives 1000 randomized network schedules per
  scenario; failures are printed as deterministic seeds for replay.

## Exit Criteria

- `hardening/loom/`, `hardening/shuttle/` no longer contain shadow code.
  Either they import production types via `cfg(loom)` / `cfg(shuttle)`, or
  the directory is removed in favor of in-tree `#[cfg(loom)] mod tests`.
- Every concurrent state machine listed under "Scope" has at least one
  model-checked invariant.
- If SPIRE remote is async, madsim lane runs in nightly CI.
- `docs/hardening.md` documents how to add a new Loom/Shuttle target,
  including the "lift then test" pattern.

## Dependencies

- Independent of Tasks 36–39.
- Pairs with Task 49 (CI governance) for nightly scheduling — these lanes
  are too slow for PR-time gating.
- Outputs may inform Task 41 (FFI safety) when concurrent code crosses the
  pgrx boundary.
