# Task 48 kickoff slice: soak harness for the quantizer OnceLock cache

## Scope

Opens Task 48 (Build Matrix, Cross-Platform, Soak, and Resource
Exhaustion — `plan/tasks/48-build-matrix-and-soak.md`).

This packet adds the first soak harness as a small, pure-Rust
scaffold: a new `ecaz stress soak-quant-cache` CLI subcommand that
drives sustained concurrent traffic against the
`ProdQuantizer::cached` `OnceLock<Mutex<HashMap<_, Arc<_>>>>` for a
configurable wall duration and emits a structured JSON summary.

No production code change; CLI-only addition. No PostgreSQL
connection required (this harness is intentionally PG-free so it
runs cleanly under the known macOS `_BufferBlocks` dyld blocker
that affects pgrx-backed paths).

Validation head: `3cc79c682` (Task 48/001 code commit, on top of
`e161976b5` Task 46/001 packet).

## What changed

- `crates/ecaz-cli/src/commands/stress/soak_quant_cache.rs` — new
  ~200-line subcommand module. Loops sub-second iterations across a
  worker-thread pool that hammers a mix of shared and private keys;
  records per-iteration ops, max shared-Arc strong count, and
  distinct shared keys observed.
- `crates/ecaz-cli/src/commands/stress/mod.rs` — registers the new
  `SoakQuantCache(SoakQuantCacheArgs)` variant on `StressCommand`.

## Why this surface first

Task 48 §Approach 3 calls out the soak harness as one of four work
areas. Picking a pure-Rust harness as the kickoff:

- Hits a real production primitive that the recent burndown work
  (Tasks 52/56.1/58.1/59) leans on heavily — every HNSW/DiskANN
  query-time encode goes through `ProdQuantizer::cached`.
- Pairs naturally with the Task 43/015 second concurrent miri test
  on the same surface. Miri proves no UB over a 53-second schedule
  sweep; this harness asserts the same canonical-Arc invariant
  holds over many-thousands of iterations on real hardware.
- Establishes the soak-harness JSON output format and CLI shape so
  future scenarios (PG-backed mixed workload, resource-exhaustion
  sweeps, build-matrix coverage) can re-use the same scaffolding.
- Runnable on macOS without the `_BufferBlocks` dyld blocker
  because it doesn't touch any pgrx callback path.

## What this slice does NOT do

Documented explicitly so the slice is judged against actual scope,
not the full Task 48 §Approach:

- **No RSS sampling and no monotonic-growth slope assertion.**
  Cross-platform RSS sampling needs a small `cfg(target_os = ...)`
  helper (`mach_task_basic_info` on macOS, `/proc/self/statm` on
  Linux) that is worth its own packet to land cleanly. The
  scaffolding here makes that follow-up a drop-in addition to the
  per-iteration record.
- No PG-backed mixed read/write soak (Task 48 §Approach 3 bullet 1).
- No resource-exhaustion harness (`ecaz dev resource-test`,
  §Approach 4).
- No build matrix CI lanes (§Approach 1) and no qemu cross-arch
  decode lane (§Approach 2).
- No `make soak DURATION=24h` Make wrapper (§Approach 6).

Each of those is a follow-up packet referenced from this one.

## Evidence

- `artifacts/cargo-check.log`: `cargo check -p ecaz-cli` — compile
  passes with no errors. (Warnings are pre-existing in the workspace
  graph and not introduced by this packet.)
- `artifacts/soak-quant-cache-5s-smoke.log`:
  `ecaz stress soak-quant-cache --duration-seconds 5 --workers 4
   --shared-keys 4 --private-keys-per-iter 2 --dim 8 --bits 4` smoke
  run. JSON output: 10 iterations, 13684 total ops, 5051ms wall,
  mean 2709 ops/sec. Per-iteration `distinct_shared_keys_observed`
  walks 1 → 3 → 4 across the run (the harness exercises every
  `shared_keys` slot once per-iter ops are sufficient).
- `artifacts/manifest.md`: command metadata and key result lines
  per the packet-local-manifest rule.

## Reviewer focus

- Subcommand signature follows the established
  `crates/ecaz-cli/src/commands/stress/*.rs` pattern (clap `Args`
  struct + free `run` fn + `serde::Serialize` summary).
- Distinctive seed namespaces (`SHARED_SEED_BASE = 0xBEEF...`,
  `PRIVATE_SEED_BASE = 0x1234...`) avoid colliding with any other
  in-process test/harness that may share the global cache.
- Iteration boundary is sub-second (`iter_budget = min(remaining,
  500ms).max(50ms)`) so the JSON output records many samples
  across the wall duration — the data shape future slope checks
  will fit against.
- The shared-Arc strong-count observation is intentionally bounded
  by an atomic-max (not an exact-count assertion) because workers
  hold the `Arc` for nondeterministic windows; the invariant that
  matters under contention is "all workers see the same canonical
  pointer", which the matched Task 43/015 miri test pins.

## Out of scope (named follow-ups)

1. Cross-platform RSS sampler + per-iteration RSS record +
   monotonic-growth slope assertion over the second-half window.
2. PG-backed mixed read/write soak harness (`ecaz stress soak-pg`).
3. Resource-exhaustion sweep (`ecaz dev resource-test`).
4. `make soak DURATION=24h` Make target wrapping this and the
   follow-up harnesses.
5. CI build-matrix lanes and qemu cross-arch decode coverage from
   Task 48 §Approach 1–2 (depends on Task 49 CI-governance
   decisions per Task 48 §Dependencies).
