# Review Request: Task 35 Priority Redirect Plan Update

Head: `10f0de0bce2ea04b0dd2208f4d1656b62eaefd54`

Scope:
- `plan/tasks/35-unsafe-quality-burndown.md`
- `reviews/task-35/021-priority-redirect-plan-update/artifacts/*`

What changed:
- Updated the canonical Task 35 packet sequence to process the reviewer
  redirect from `reviews/task-35/004-master-unsafe-burndown-plan/feedback/2026-05-18-01-reviewer.md`.
- Stopped scheduling new HNSW packets ahead of the higher-priority coverage
  gaps.
- Moved the next slices to:
  1. quant / RABITQ-adjacent SIMD (`src/quant/hadamard.rs`, `src/quant/prod.rs`);
  2. IVF planner/options/admin, then page, scan, and maintenance paths;
  3. SPIRE small callbacks, relation/vacuum/cost, storage/page, snapshots,
     CustomScan, and DML frontdoor;
  4. HNSW residuals after those surfaces reach better parity.

Current state:
- Baseline report still shows 3,050 entries across 101 files.
- `src/am/ec_hnsw/graph.rs` remains at 35 baseline entries; the abandoned local
  HNSW tail slice was not included in this packet.
- The unsafe audit currently fails on `src/am/ec_ivf/page.rs` exact-line drift
  introduced by the incoming main work. That drift is now aligned with the
  reviewer redirect because the next concrete burndown target is in IVF/quant,
  and the next regular burndown packet should update the baseline only while
  producing a net decrease.

Validation:
- `make unsafe-baseline-report`
  - artifact: `artifacts/unsafe-baseline-report.log`
  - result: 3,050 baseline entries across 101 files.
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/unsafe-audit.log`
  - result: fails on current `src/am/ec_ivf/page.rs` line drift.
- `git diff --check`
  - artifact: `artifacts/git-diff-check.log`
  - result: passes.

Tests skipped:
- No Rust behavior changed; this is a task-plan update and review packet only.
