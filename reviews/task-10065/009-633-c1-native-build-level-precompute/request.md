# Review Request: Native Build Level Precompute

Current head: `cddb80f`

Scope:
- `src/am/ec_hnsw/build.rs`
- `src/am/ec_hnsw/build_parallel.rs`

Context:
- Packet 632 revised ADR-048 to pursue concurrent HNSW graph insertion into a
  DSM node array protected by per-node LWLocks.
- The first required pre-assembly step is making node levels explicit before
  graph assembly starts, so the future concurrent path can choose a fixed entry
  node and allocate per-node neighbor slots up front.

Change:
- Added `EcHnswBuildGraphAssembly::ConcurrentDsm` as an opt-in planning variant,
  without changing the default build plan.
- Added `NativeBuildLevels` and `precompute_native_build_levels`.
- Updated the serial native builder to consume the precomputed level vector
  while preserving the existing insertion order, entry tracking, graph topology,
  and page-staging behavior.
- Added unit coverage for fixed-entry selection over precomputed levels.

Validation:
- `cargo test native_build_levels`
- `cargo test`
- `cargo pgrx test pg18`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Review focus:
- Whether `NativeBuildLevels` exposes the right metadata for the next DSM graph
  allocation slice (`levels`, first max-level `entry_idx`, `max_level`).
- Whether reusing the precomputed level vector in serial build is low-risk and
  behavior-preserving.
- Whether keeping `ConcurrentDsm` present but non-default is the right gating
  surface before DSM allocation and worker insertion are implemented.
