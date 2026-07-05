# Review Request: IVF Admin Cost Options Safety

Head: `fdf7a8ee3c8a67aa402aa7468e8d4e45033eb2f7`

Scope:
- `src/am/ec_ivf/admin.rs`
- `src/am/ec_ivf/cost.rs`
- `src/am/ec_ivf/options.rs`
- `scripts/unsafe_comment_baseline.txt`
- `reviews/task-35/024-ivf-admin-cost-options-safety/artifacts/*`

What changed:
- Documented the small IVF admin, planner-cost, and reloption unsafe
  boundaries.
- Covered PostgreSQL AM callback guards for cost/tree-height/strategy
  translation.
- Covered IVF reloption registration, reloption string offset reads, and
  `rd_options` layout casting.
- Covered admin diagnostic reads for metadata, directory drift, block counts,
  reltuples, and posting-page ownership summaries.

Baseline accounting:
- Global baseline: 2,977 -> 2,955.
- `src/am/ec_ivf/admin.rs`: 10 -> 0.
- `src/am/ec_ivf/cost.rs`: 4 -> 0.
- `src/am/ec_ivf/options.rs`: 8 -> 0.
- Combined target files: 22 -> 0.

Unsafe sites:
- Removed no unsafe blocks in this slice.
- Added nearby `// SAFETY:` contracts for the callback, reloption, and admin
  diagnostic unsafe blocks.

Validation:
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-audit-after.log`
  - result: passes.
- `make unsafe-baseline-report`
  - artifacts: `artifacts/unsafe-baseline-report-before.log`,
    `artifacts/unsafe-baseline-report-after.log`
  - result: 2,977 -> 2,955.
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
- Whether the reloption comments sufficiently tie raw offsets and C-string reads
  to the `EcIvfReloptions` layout registered by `ec_ivf_amoptions`.
- Whether admin diagnostic relation/page reads are documented at the right
  layer, given the larger IVF page substrate remains scheduled separately.
