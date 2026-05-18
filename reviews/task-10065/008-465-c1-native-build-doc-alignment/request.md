# Review Request: Native Build Doc Alignment

Current head: `4b637b3`

Scope:
- `README.md`
- `docs/PG18_UPGRADE_PLAN.md`
- `docs/architecture.md`
- `plan/plan.md`
- `spec/adr/ADR-016-pg18-primary-target.md`
- `spec/functional/FR-008-hnsw-build.md`
- `spec/functional/FR-021-parallel-build.md`
- `spec/spec.md`

Problem:
- The live docs and spec still described bulk HNSW build as if production Ecaz
  used `hnsw_rs`.
- That was no longer true after the native in-crate build path landed on
  `main`, so the docs were lagging the actual code and task state.
- The stale wording was especially misleading in the build requirements,
  parallel-build planning text, and top-level spec/system description.

What changed:
- Removed the stale `hnsw_rs` production-path wording from the live docs/spec
  that describe the current build architecture.
- Rewrote `FR-008` to describe the current native build flow:
  - heap scan and build-tuple collection
  - native graph construction via Ecaz-owned build state
  - page serialization from native build output
- Rewrote `FR-021` and the PG18 parallel-build planning text so the current
  limitation is described accurately:
  - parallel build is still off today
  - the native builder is still leader-only
  - future work is to parallelize Ecaz's own builder rather than replace
    `hnsw_rs`
- Updated the architecture/spec overview text and source-tree snapshot so the
  top-level repository description matches the current `am/common`,
  `am/ec_hnsw`, `storage/`, and native-builder layout.
- Dropped the stale `hnsw_rs` reference entry from the current README/spec
  references.

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Whether the touched live docs/spec now describe the current native HNSW build
  path accurately
- Whether the updated `FR-008` / `FR-021` wording stays aligned with the
  current implementation without overcommitting future parallel-build design
- Whether the top-level spec and architecture text now reflects the post-split
  module layout and owned native builder cleanly
- Whether any still-live doc/spec surface is still implying `hnsw_rs` remains
  part of the production build path
