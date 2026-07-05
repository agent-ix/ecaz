# Review Request: Task 28 IVF Bulk Assignment Staging

Scope: Phase 3 bulk-assignment checkpoint. Populated `ec_ivf` builds now
assign collected rows to trained centroids and stage centroid, posting-list,
directory, and metadata records in memory before the still-explicit
populated-write gate.

Task: `plan/tasks/28-ivf-access-method.md` Phase 3

Branch: `task28-ivf`

Head SHA: `770b9f67effd992e7f9ecf3fbc3b7c5646d8968b`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo test`
- `cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `git diff --check`

Final validation highlights:

- `cargo test`: main pg18 lib suite reported 580 passed, 0 failed, 4 ignored;
  proptests, recall smoke, size assertions, and doc tests also passed.
- `cargo pgrx test pg17`: main pg17 lib suite reported 577 passed, 0 failed,
  4 ignored; proptests, recall smoke, size assertions, and doc tests also
  passed.
- Clippy completed cleanly with `-D warnings`.

## Summary

This slice adds a narrow in-memory build plan after centroid training:

- Adds `IvfBuildPlan` to hold staged data pages, populated metadata, centroid
  tuple TIDs, directory tuple TIDs, posting tuple TIDs grouped by list, and
  list-directory entries.
- Assigns every collected heap tuple to its nearest trained spherical centroid
  through `training::assign_vector_to_centroid`.
- Writes centroid tuples, per-row posting tuples, and per-list directory tuples
  into a `DataPageChain` using the Phase 2 IVF page codecs.
- Stages metadata with trained dimensions, resolved `nlists`, training
  version, centroid head, directory head, and total live tuple count.
- Records empty-list counts through directory entries with invalid head/tail
  block refs and zero live count.
- Keeps populated on-disk writes gated, but now reports staged centroid,
  directory, posting, empty-list, and data-page counts in the explicit error.
- Adds focused unit coverage for list assignment counts, empty-list directory
  refs, metadata heads, and centroid/directory/posting readback from staged
  pages.

## Review Focus

Please review for:

- Whether staging postings list-by-list is the right boundary before physical
  writes, given the current directory stores block refs rather than tuple refs.
- Whether directory `head_block` / `tail_block` is sufficient for v1 list
  scanning, or whether the next slice should change the directory shape before
  on-disk writes are enabled.
- Whether one posting tuple per heap row is acceptable for the first build
  path, or whether duplicate heap TID coalescing should happen before the
  physical write checkpoint.
- Whether metadata should store resolved training-sample count or additional
  source/quantizer fields before the writer lands.
- Whether the staged-plan error message contains the right operational detail
  while populated writes remain intentionally blocked.

## Non-Goals

This packet does not write populated IVF pages to disk, perform WAL-safe
metadata or list-directory updates, implement build stats beyond staged counts,
enable scans over populated IVF indexes, implement live insert, vacuum, planner
costing, or make any measurement claim.
