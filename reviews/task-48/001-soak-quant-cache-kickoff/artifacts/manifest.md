# Packet 001 — Task 48 kickoff: soak harness for quantizer OnceLock cache

## Head

- Task bucket: `reviews/task-48/`
- Packet path: `reviews/task-48/001-soak-quant-cache-kickoff/`
- Validation head SHA: `3cc79c682` (Task 48/001 code commit; sits
  on top of Task 43/015 and Task 46/001 commits from this session)
- Branch: `main`
- Surface under validation: pure-Rust quantizer cache
  (`src/quant/prod.rs:72-127` — `OnceLock<Mutex<HashMap<_, Arc<_>>>>`
  backing `ProdQuantizer::cached`). Same surface as Task 43/015,
  exercised from a different angle (wall-clock soak vs. Miri
  schedule sweep).
- Storage format / fixture: N/A — pure-Rust CLI harness, no on-disk
  format, no PostgreSQL connection.
- Rerank mode / lane: N/A — soak/longevity check, not a recall or
  latency benchmark.
- Surface isolation: single-process CLI binary; no PG backend; no
  shared-table contention. Uses distinctive seed namespaces
  (`SHARED_SEED_BASE = 0xBEEF...`, `PRIVATE_SEED_BASE = 0x1234...`)
  to avoid collision with any other in-process test/harness that
  shares the global cache.

## Subcommand added

- `crates/ecaz-cli/src/commands/stress/soak_quant_cache.rs` —
  `ecaz stress soak-quant-cache` subcommand.
- `crates/ecaz-cli/src/commands/stress/mod.rs` — registers
  `SoakQuantCache(SoakQuantCacheArgs)` on `StressCommand`.

## Artifacts

### cargo-check.log

- Command: `cargo check -p ecaz-cli`
- Timestamp: 2026-05-25
- Result: compile pass, `Finished `dev` profile ... in 12m 07s`,
  exit code 0. One pre-existing warning in `ecaz` lib
  (`unused import: load_relation_local_store_config`); no new
  warnings introduced by this packet.

### soak-quant-cache-5s-smoke.log

- Command:
  `ecaz stress soak-quant-cache --duration-seconds 5 --workers 4
   --shared-keys 4 --private-keys-per-iter 2 --dim 8 --bits 4`
- Timestamp: 2026-05-25
- Result: JSON summary captured to stdout (per
  `serde_json::to_string_pretty`). Actual smoke values:
  - `iterations_completed: 10`
  - `total_ops: 13684`
  - `wall_elapsed_ms: 5051`
  - `mean_ops_per_sec: 2709.10`
  - per-iteration `distinct_shared_keys_observed` walks 1 → 3 → 4
    across the run (= configured `shared_keys: 4` once the per-iter
    op budget covers the full key range)
  - per-iteration `shared_arc_strong_count_max` ∈ {2, 3, 4} under
    4-worker contention — healthy contention signal, not pinned to a
    single value.

## Key result lines cited by request.md

- `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 12m 07s` (cargo-check.log)
- `EXIT=0` (cargo-check.log)
- JSON `iterations_completed > 0` and `total_ops > 0` from the
  soak smoke log demonstrate the harness completes its wall-budget
  loop and records per-iteration samples.

## Out of scope (documented in request.md follow-ups)

- Cross-platform RSS sampler + slope assertion
- PG-backed mixed read/write soak
- Resource-exhaustion sweeps
- `make soak DURATION=24h` wrapper
- CI build matrix lanes + qemu cross-arch decode lane
