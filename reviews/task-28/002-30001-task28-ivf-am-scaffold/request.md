# Review Request: Task 28 IVF AM Scaffold

Scope: Phase 1 scaffold only. Registers the optional `ec_ivf` access
method, adds reloptions/GUC plumbing, and wires AM callbacks to explicit
not-implemented boundaries before storage/build logic starts.

Task: `plan/tasks/28-ivf-access-method.md` Phase 1

Branch: `task28-ivf`

Head SHA: `55239016be0637feafa3c314fdf5f806b4e3f3d7`

Owner: coder2

Files:

- `src/am/ec_ivf/mod.rs`
- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/routine.rs`
- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/insert.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/vacuum.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/training.rs`
- `src/am/mod.rs`
- `sql/bootstrap.sql`
- `plan/tasks/28-ivf-access-method.md`
- `src/quant/rabitq.rs`

Validation:

- `git diff --cached --check`
- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Summary

This slice adds the first code scaffold for the IVF access method:

- `ec_ivf` is registered beside `ec_hnsw`.
- SQL bootstrap defines `ec_ivf_handler`, `CREATE ACCESS METHOD ec_ivf`,
  and `tqvector` / `ecvector` inner-product operator classes for the new AM.
- Reloptions are registered for `nlists`, `nprobe`,
  `training_sample_rows`, `seed`, `storage_format`, and `rerank`.
- `ec_ivf.nprobe` exists as the session override GUC.
- AM routine callbacks are wired, but populated build/insert/scan/vacuum
  paths intentionally raise explicit `ec_ivf ... is not implemented yet`
  errors.
- Planner cost estimate is gated with effectively unusable costs until the
  AM has real scan behavior.
- The task plan marks module layout, SQL bootstrap, and skeleton callbacks
  complete; empty-index behavior remains the next functional slice.

The `src/quant/rabitq.rs` change is an incidental clippy cleanup required
by the checkpoint gate after current Rust/Clippy flagged two range-index
loops.

## Review Focus

Please review for:

- Whether the module split under `src/am/ec_ivf/` is the right starting
  surface before page metadata and training code land.
- Whether exposing `CREATE ACCESS METHOD ec_ivf` before functional
  `CREATE INDEX` support is acceptable given that callbacks fail loudly.
- Whether the reloption names and defaults leave enough room for
  TurboQuant, PqFastScan, RaBitQ, and later storage profiles.
- Whether `ec_ivf.nprobe` as the only initial IVF GUC is the right runtime
  override boundary.
- Whether reusing `tqvector_ip_ops` / `ecvector_ip_ops` for `USING ec_ivf`
  is acceptable on the installed PostgreSQL versions.
- Whether the all-max cost estimate is sufficient planner gating during the
  scaffold phase.
- Whether the small RaBitQ iterator cleanup is acceptable in this checkpoint
  or should be split if reviewers want stricter feature-only commits.

## Non-Goals

This packet does not implement IVF metadata pages, centroid training,
posting lists, empty-index scans, candidate scoring, or planner costing.
Those are tracked as subsequent Phase 1/2/3 slices.
