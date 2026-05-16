# Task 35: Unsafe Quality Burndown

Status: **active** — Task 34 installed the local-first hardening lanes and
grandfathered the historical unsafe-comment debt. This task owns burning that
baseline down to zero through reviewed subsystem packets.

## Scope

Review every entry in `scripts/unsafe_comment_baseline.txt`. Each legacy unsafe
site must be removed, hidden behind a safe wrapper that carries the invariant,
or documented with a specific nearby `// SAFETY:` comment. Placeholder comments
or restatements of the code do not satisfy this task.

The current baseline is intentionally treated as quality debt, not as an
accepted permanent exception to `NFR-004`.

## Baseline Accounting

Use `make unsafe-baseline-report` before and after each burndown packet. Packet
artifacts should include the raw before/after report output when citing counts.

Each packet request should record:

- starting and ending baseline entry count,
- files and subsystem covered,
- unsafe sites removed, wrapped, or documented,
- any invariant that remains high-risk or requires reviewer attention,
- validation commands run or explicitly skipped under the coder test policy.

Baseline entries may only decrease. If a packet adds a new undocumented unsafe
site, the packet is blocked unless the review request calls out the temporary
reason and a reviewer accepts it.

## Packet Sequence

Work in narrow, reviewable slices, roughly 100-300 baseline entries when the
subsystem permits it:

1. `src/lib.rs` pgrx SQL and FFI entrypoints.
2. `ec_hnsw` scan, scan-debug, build, insert, vacuum, and shared-state paths.
3. `ec_ivf` page, scan, build, insert, vacuum, and admin paths.
4. `ec_spire` DML frontdoor, page, coordinator, storage, CustomScan, build,
   update, and vacuum paths.
5. `ec_diskann` routine, insert, build, and scan-state paths.
6. common AM support, quant helpers, pgstat shims, and standalone stubs.
7. test-only unsafe sites, after production paths are reviewed.

Within each subsystem, prefer deletion or safe wrappers over comments when the
unsafe pattern repeats and a wrapper can encode the real contract.

## Validation

Every packet must run:

- `bash scripts/check_unsafe_comments.sh`
- `make unsafe-baseline-report`
- `git diff --check`

Run focused `cargo check`, `cargo test`, or PG18 validation when the packet does
more than add comments, when it changes wrappers around PostgreSQL callbacks,
or when it touches page/WAL/scan/build/vacuum/DML behavior.

## Exit Criteria

- `scripts/unsafe_comment_baseline.txt` is empty.
- `make audit-unsafe` passes without grandfathered entries.
- `TC-036` in Task 09 is marked complete.
- `NFR-004` safety language and docs no longer describe legacy unsafe debt as
  accepted baseline state.
