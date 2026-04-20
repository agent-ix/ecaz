# Review Request: PG18 Shared-Infra Merge And Wiring

Current head: `c13a6aa`

Scope:
- `Cargo.toml`
- `build.rs`
- `csrc/pg18_pgstat_shim.c`
- `Makefile`
- `.github/workflows/ci.yml`
- `README.md`
- `docs/pg18.md`
- `plan/tasks/19-pg18-completion.md`
- `spec/functional/FR-012-sql-bootstrap.md`
- `spec/functional/FR-025-custom-statistics.md`
- `spec/functional/FR-026-pg18-module-identity.md`
- `spec/functional/FR-027-pgrx-pg18-upgrade.md`
- `spec/tests.md`
- `src/am/common/cost.rs`
- `src/am/common/explain.rs`
- `src/am/common/stats.rs`
- `src/am/common/stream.rs`
- `src/am/ec_hnsw/graph.rs`
- `src/am/ec_hnsw/mod.rs`
- `src/am/ec_hnsw/routine.rs`
- `src/am/ec_hnsw/scan.rs`
- `src/am/ec_hnsw/shared.rs`
- `src/lib.rs`
- `src/pg18_pgstat_shim.rs`

Problem:
- `main` had already split the AM into `common` and `ec_hnsw` modules while `origin/pg18`
  still carried older PG18-upgrade work against earlier boundaries.
- The PG18 plan still had shared-infrastructure work open: AM callback wiring, EXPLAIN hook
  registration, staged statistics plumbing, ReadStream integration points, module identity, and
  Cargo / CI defaults.
- The remaining scope here is intentionally limited to shared infrastructure. No pipeline-specific
  enablement, tuning, measurement, or storage-format feature work is included.

What changed:
- Merged `origin/pg18` into the current `main` line and rebased the PG18 work onto the split
  `common` / `ec_hnsw` module layout.
- Flipped the extension identity and Cargo setup to PG18-primary / PG17-fallback:
  - `Cargo.toml` default feature is now `pg18`
  - PG14-PG16 feature flags are dropped
  - package / control-file version now matches `0.1.1`
  - CI now initializes both pg17 and pg18 explicitly and runs separate pgrx jobs
- Wired PG18-facing AM callbacks in `src/am/ec_hnsw/routine.rs` and `src/am/common/cost.rs`:
  - `amconsistentordering = true`
  - `amgettreeheight`
  - `amtranslatestrategy`
  - `amtranslatecmptype`
  - planner/cost snapshots now distinguish the PG18 callback path from the PG17 metadata fallback
- Finished EXPLAIN hook registration and per-scan counter plumbing:
  - `_PG_init()` now registers the PG18 EXPLAIN option/hook path
  - hook logic lives in `src/am/common/explain.rs`
  - scan execution updates the staged counter sites and exposes counters through the existing
    snapshot/test surfaces
- Finished staged PG18 statistics plumbing:
  - `_PG_init()` now calls `register_pg18_stats()`
  - `tqvector_stats()` is live on PG18
  - a PG18-only C shim now owns the `pgstat_internal.h` boundary and registers a fixed custom
    pgstat kind during shared preload
  - `tqvector_stats()` reads the shared pgstat snapshot when that registration is active, and
    otherwise falls back to the existing backend-local counters in non-preloaded sessions
  - diagnostics snapshots now report the preload/runtime blocker instead of the old bindings/shim
    code blocker
- Finished ReadStream / async-I/O shared wiring:
  - pure callback/state helpers in `src/am/common/stream.rs` now map to PG18 callback signatures
  - scan graph prefetch, linear fallback reads, and vacuum tuple counting all have PG18-specific
    ReadStream attach points
  - PG17 keeps the legacy buffer-read fallback path
- Updated docs/spec/task text so the staged PG18 boundary is accurate after the merge.

Live now:
- PG18 AM callback surface for tree height and strategy/compare translation
- PG18 EXPLAIN option registration and per-node hook registration
- PG18 shared pgstat registration path via `shared_preload_libraries`
- PG18 `tqvector_stats()` SQL surface, with shared-snapshot reads when preloaded and backend-local
  fallback otherwise
- PG18 ReadStream-backed graph-neighbor prefetch, linear fallback block reads, and vacuum tuple
  counting code paths
- PG18 module identity / SQL surface expectations in tests and docs
- PG17 fallback build/test/lint path

Still gated:
- Shared pgstat activation still requires runtime preload configuration:
  `custom pgstat kind registration requires loading tqvector via shared_preload_libraries on PG18 and restarting PostgreSQL`
- This machine still cannot run PG18 build/test/lint because `pgrx` does not manage a PG18 install.
- No pipeline-specific or storage-format-specific PG18 enablement was added in this slice.

Validation:
- Passed:
  - `cargo test --no-default-features --features pg17`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
  - `bash scripts/run_pgrx_pg17_test.sh`
- Attempted but blocked by local environment:
  - `cargo test`
  - `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - `cargo pgrx test pg18`
- The PG18 failure mode on this machine is consistent and environment-specific:
  `Postgres 'pg18' is not managed by pgrx`
  `~/.pgrx/config.toml` only advertises pg17, and the local `~/.pgrx/18.3` tree does not contain a
  built `pgrx-install/bin/pg_config`.

Review focus:
- Whether the PG18 callback wiring is attached at the right shared-AM seams without leaking
  pipeline-specific behavior into this branch
- Whether the EXPLAIN hook registration and chaining logic are correct and safe under PG18
- Whether the staged stats story is clear now that the shared pgstat path exists but still depends
  on preload-time activation, with backend-local fallback left in place for ordinary sessions
- Whether the ReadStream integration points sit in the right shared/module boundaries and preserve
  PG17 fallback behavior
- Whether the docs/spec/task updates accurately describe what is live versus still blocked

Questions to answer:
- Are any of the PG18 callback / EXPLAIN / ReadStream hooks attached too deep in `ec_hnsw` runtime
  code when they should stay in `common` shared infrastructure?
- Is the shim boundary the right long-lived place for `pgstat_internal.h`, or should more of the
  registration/snapshot logic move out of C once `pgrx` exposes better PG18 internals?
- Is the shared-snapshot plus backend-local fallback behavior the right contract for
  `tqvector_stats()` until preload-aware PG18 validation is available in this repo?
