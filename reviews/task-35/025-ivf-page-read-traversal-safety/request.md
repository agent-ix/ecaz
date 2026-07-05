# Review Request: IVF Page Read Traversal Safety

Head: `74da472bd211e955e2f7835621fae5f600ceb53a`

Scope:
- `src/am/ec_ivf/page.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/025-ivf-page-read-traversal-safety/artifacts/*`

What changed:
- Documented the first IVF page read/traversal layer.
- Covered centroid, list-directory, PQ-codebook, and posting-list reader
  entrypoints.
- Covered validated posting block range traversal for collection, TID
  collection, rewrite, block-sequence visits, and posting-ref visits.
- Left PG18 read-stream internals and WAL/page mutation helpers for later
  focused IVF page packets.

Baseline accounting:
- Global baseline: 2,955 -> 2,942.
- `src/am/ec_ivf/page.rs`: 134 -> 121.

Unsafe sites:
- Removed no unsafe blocks in this slice.
- Added nearby `// SAFETY:` contracts for read traversal unsafe blocks that
  delegate into lower page/buffer helpers.

Validation:
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passes.
- `make unsafe-baseline-report`
  - artifacts: `artifacts/unsafe-baseline-report-before.log`,
    `artifacts/unsafe-baseline-report-after.log`
  - result: 2,955 -> 2,942.
- `cargo fmt --all`
  - artifact: `artifacts/cargo-fmt.log`
  - result: passes; unrelated fmt churn was restored before commit.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18-bench.log`
  - result: passes with the existing unused-import warnings.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passes.

Reviewer focus:
- Whether the entrypoint comments correctly rely on lower page helpers for
  bounds/tag validation without over-claiming page-layout safety.
- Whether the remaining IVF page entries should continue as separate
  read-stream, append/WAL, rewrite/delete, debug walk, and metadata packets.
