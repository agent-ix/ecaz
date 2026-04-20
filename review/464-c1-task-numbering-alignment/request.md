# Review Request: Task Numbering Alignment

Current head: `ae0545f`

Scope:
- `plan/tasks/README.md`
- `plan/tasks/archive/README.md`
- `plan/tasks/archive/05-build-and-scan.md`
- `plan/tasks/archive/06-vacuum-and-insert.md`
- `plan/tasks/archive/07-simd-and-benchmarks.md`
- `plan/tasks/archive/08-safety-and-ci.md`
- `review/30-plan-and-spec-backfill/request.md`
- `review/127-admin-snapshot-for-planner-and-insert-stats.md`
- `review/10001-ef-search-control-surface-and-planner-gate-scaffolding/request.md`
- `review/10002-admin-snapshot-for-planner-and-insert-stats/request.md`
- `review/10003-explain-snapshot-for-planner-gate/request.md`
- `review/421-c1-adr030-v2-final-local-landing-proof-artifact/feedback/reviewer-1.md`

Problem:
- The live task tree still had four coarse pre-lane task files occupying the
  same numeric slots as the current split task set:
  - `05-build-and-scan.md`
  - `06-vacuum-and-insert.md`
  - `07-simd-and-benchmarks.md`
  - `08-safety-and-ci.md`
- Those files were historical snapshots, but leaving them at the top level made
  the active task numbering ambiguous.
- `plan/tasks/README.md` also pointed DiskANN at the wrong filename
  (`17-diskann.md` instead of `17-diskann-access-method.md`).

What changed:
- Moved the four superseded coarse task files under `plan/tasks/archive/` so
  the live top-level task inventory is unique again.
- Marked each moved file as an archived legacy snapshot and pointed it at its
  live successor task file(s).
- Added `plan/tasks/archive/README.md` to make the archive role explicit.
- Updated `plan/tasks/README.md` to:
  - document the archive section
  - keep only the live split tasks in the top-level numbered list
  - fix the DiskANN filename reference
  - refresh the visible status text for Task 07 and Task 11 while touching the
    index
- Repointed the small set of in-repo review/request references that still named
  the old top-level coarse-task paths, so the cleanup does not leave dead links.

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Whether the top-level live task numbering is now unambiguous
- Whether archiving the pre-lane coarse task files is the right cleanup boundary
  versus renumbering or deleting them
- Whether the touched historical review/request docs now point at the archived
  paths cleanly without changing their substantive historical claims
- Whether the `plan/tasks/README.md` DiskANN and status text now reflects the
  real current task tree
