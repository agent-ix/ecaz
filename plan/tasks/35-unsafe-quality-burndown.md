# Task 35: Unsafe Quality Burndown

Status: **complete** as of 2026-05-19 (final closeout commit
`5bc35c9a`; baseline is empty). Task 34 installed the local-first
hardening lanes and grandfathered the historical unsafe-comment debt;
Task 35 burned that baseline down to zero through reviewed subsystem
packets.

Final accounting:

- `scripts/unsafe_comment_baseline.txt` is empty
  (`bash scripts/unsafe_baseline_report.sh` reports
  `entries: 0`, `files: 0`).
- ~3,397 baseline entries cleared across ~120 code-bearing packets.
- AM closeouts on file: SPIRE (083), HNSW (104), DiskANN / `src/am`
  residual (107), IVF retroactive (122).
- Top-level closeout: 121.
- Test-only sweep (packets 108–120) cleared 499 entries and
  prototyped the AM callback wrapper macro pattern in 10 test files
  — Task 50 has working prototypes to mine from.

Follow-on structural-reduction work is tracked by Task 50
(`50-unsafe-structural-reduction.md`).

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
subsystem permits it. The initial HNSW scan/graph packets established the
pattern for production read-path comments and wrappers; new packets should now
pivot to the reviewer-prioritized gaps before returning to HNSW residuals.

1. Quant / RABITQ-adjacent SIMD:
   - `src/quant/hadamard.rs`
   - `src/quant/prod.rs`
2. IVF planner, options, and admin boundaries:
   - `src/am/ec_ivf/cost.rs`
   - `src/am/ec_ivf/options.rs`
   - `src/am/ec_ivf/admin.rs`
3. IVF page and storage substrate:
   - `src/am/ec_ivf/page.rs`
4. IVF scan and posting-list traversal:
   - `src/am/ec_ivf/scan.rs`
5. IVF build, insert, and vacuum maintenance paths:
   - `src/am/ec_ivf/build.rs`
   - `src/am/ec_ivf/insert.rs`
   - `src/am/ec_ivf/vacuum.rs`
6. SPIRE small callback and coordinator surfaces:
   - `src/am/ec_spire/scan/callbacks.rs`
   - `src/am/ec_spire/coordinator/lifecycle.rs`
   - `src/am/ec_spire/coordinator/diagnostics.rs`
   - small `src/am/ec_spire/remote_candidates/*` files
7. SPIRE relation, vacuum, and cost surfaces:
   - `src/am/ec_spire/scan/relation.rs`
   - `src/am/ec_spire/vacuum/mod.rs`
   - `src/am/ec_spire/cost/mod.rs`
8. SPIRE storage and page substrate:
   - `src/am/ec_spire/storage/relation_store.rs`
   - `src/am/ec_spire/page.rs`
9. SPIRE coordinator snapshots:
   - `src/am/ec_spire/coordinator/snapshots.rs`
   - `src/am/ec_spire/coordinator/hierarchy_snapshots.rs`
10. SPIRE CustomScan surfaces:
    - `src/am/ec_spire/custom_scan/*`
11. SPIRE DML frontdoor:
    - `src/am/ec_spire/dml_frontdoor/mod.rs`
12. HNSW residual build, insert, vacuum, routine, and shared-state paths.
13. `ec_diskann` routine, insert, build, and scan-state paths.
14. common AM support, pgstat shims, standalone stubs, and remaining shared
    helper surfaces.
15. test-only unsafe sites, after production paths are reviewed.

Within each subsystem, prefer deletion or safe wrappers over comments when the
unsafe pattern repeats and a wrapper can encode the real contract.

If the site can plausibly be deleted by an in-flight structural refactor (Task
40 lifted concurrency modules, Task 41 PG resource wrappers, Task 43 expanded
Miri/cargo-careful proof coverage, or equivalent), defer the packet that would
annotate it until the refactor lands or is abandoned.

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
