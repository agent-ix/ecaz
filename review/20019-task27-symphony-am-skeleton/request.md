# Review Request: Task 27 Slice 3 — Symphony AM Skeleton

Scope: Phase-1 scaffold only. Registers `symphony` as a third access
method, adds the ADR-041 module layout under `src/am/symphony/`, and
freezes the initial handler/callback surface without landing any graph,
page-codec, build, insert, scan, or vacuum behavior yet.

Task: `plan/tasks/27-symphony-access-method.md` Phase 1
("Module scaffold").

Branch: `task27-symphony-stage2-phase0-oracle` (slice 3 builds on
`42394d4`).

Files in scope:
- `src/am/mod.rs`
- `src/am/symphony/{build,graph,insert,mod,page,routine,scan,vacuum}.rs`
- `sql/bootstrap.sql`
- `src/quant/rabitq.rs`

Validation at the code checkpoint:
- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## What landed

### 1. Third AM registration surface

`src/am/mod.rs` now wires a new `symphony` module beside `ec_hnsw`,
and `sql/bootstrap.sql` registers:

- `symphony_handler`
- `CREATE ACCESS METHOD symphony`
- `tqvector_symphony_ip_ops`
- `ecvector_symphony_ip_ops`

This gives later slices a stable SQL/bootstrap seam without reusing
`ec_hnsw` names or handler state.

### 2. ADR-041 module scaffold

The new module tree is present:

- `build.rs`
- `insert.rs`
- `scan.rs`
- `page.rs`
- `graph.rs`
- `vacuum.rs`
- `routine.rs`

Each behavior entry point currently routes through a shared
`not_implemented("<callback>")` helper in `src/am/symphony/mod.rs`.
That is intentional for this slice: callers fail loudly rather than
silently inheriting `ec_hnsw` behavior.

### 3. AM routine stub

`src/am/symphony/routine.rs` provides:

- `symphony_handler`
- `pg_finfo_symphony_handler`
- `IndexAmRoutine` wiring for build / insert / scan / vacuum callbacks
- a permissive `amvalidate` stub

The routine advertises the same broad ordered-vector shape expected of
the future AM (`amcanorderbyop = true`, no bitmap path, no backward
scan), but none of the callbacks implement behavior yet.

### 4. First Symphony-owned page constant

`src/am/symphony/page.rs` adds:

- `INDEX_FORMAT_V5_SYMPHONY`

No tuple codec is implemented in this slice. The constant exists so
later page-layout work can target a dedicated on-disk format instead of
mutating `ec_hnsw` format IDs in place.

### 5. Small RaBitQ cleanup needed to keep the checkpoint green

`src/quant/rabitq.rs` picks up a narrow cleanup so the new checkpoint
passes the required lint lane after rebasing onto the task-25 code:

- replace two index-based loops with iterator/enumerate forms
- accept rustfmt-only line wrapping around nearby expressions

There is no intended algorithmic change in this file for this slice.

## What this slice intentionally does NOT do

- no Symphony page encoder / decoder
- no centered-code adjacency storage
- no build path
- no insert path
- no scan state or traversal
- no vacuum logic
- no reloptions or GUC surface
- no tests specific to Symphony yet

The target here is only to reserve the module, bootstrap, and handler
surfaces so later Stage-2 slices can land one mechanism at a time.

## Review focus

Please check three things only:

1. The SQL/bootstrap registration is sufficient and non-conflicting for
   a third AM.
2. The module split follows ADR-041 cleanly without reaching back into
   `ec_hnsw`.
3. The stub surface fails fast in a way that will not hide accidental
   execution before later slices implement behavior.

## Closing

This slice is the narrowest possible Phase-1 opener: create the
Symphony AM identity, reserve its module boundaries, and keep every
runtime callback explicitly unimplemented until the page codec and
search mechanics arrive in later packets.
