# Task 109: Standardized ecaz Bench Sweep Config

Status: proposed (2026-06-16)
Owner: unassigned
Priority: 3 (benchmark-workflow consolidation)

## Why

Every task hand-authors a fresh `SuiteConfig` JSON (dozens under `benchmarks/`
plus reference configs in `crates/ecaz-cli/suites/`), so they drift in sweep
grids and scale sets. The runner (`ecaz bench suite`, FR-038) is fine and
unchanged — the gap is purely a missing canonical config + convention. Seed
per-lane configs already exist at
`crates/ecaz-cli/suites/current/{m5-local,intel-local,aws-intel,aws-graviton}.json`.

## Scope

1. Define and commit canonical per-lane standard suite configs, building on the
   `current/` seeds. Each covers the standard profiles (`ec_hnsw`, `ec_ivf`,
   `ec_diskann`, `ec_spire`) × standard scales (10k/50k/100k/1m) × standard
   steps (load/recall/latency/storage), using the per-profile `default_sweep`
   already in `crates/ecaz-cli/src/profiles.rs`.
2. Document them as **THE standard ecaz sweep** in `CLAUDE.md` (Benchmark Runner
   section) and `crates/ecaz-cli/README.md`.
3. Convention: tasks **run the standard lane config as-is**; only hand-author a
   suite when there's a specific reason, stated in the packet manifest.
4. (Follow-up note only, not this cut) audit/retire redundant per-task suites.

No include/override mechanism and no generator — tasks reference and run the
committed canonical config directly.

## Acceptance criteria

- `ecaz bench suite --config crates/ecaz-cli/suites/current/<lane>.json
  --dry-run` expands to the full standard profile × scale × step matrix with the
  `profiles.rs` default sweeps, for each lane.
- `CLAUDE.md` and `crates/ecaz-cli/README.md` document the standard config and
  the "run-as-is, justify custom" convention.

## Coordination

- Pairs with Task 108 (comparator bench unification). Config + docs only, no
  Rust code changes to the runner.
