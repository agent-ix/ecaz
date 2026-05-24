# Review Request: SPIRE DML Baserel View Handoff + Relation Guardrail

Task: 50 unsafe burndown

Commits under review:

- `35ef01ca` - `Make SPIRE DML baserel handoff helpers safe`
- `572ab705` - `Scope SPIRE DML baserel view to callback`
- `69893cd1` - `Warn on safe Relation helper signatures`

## Summary

This packet covers the current WIP completion after the soundness-audit feedback about raw PostgreSQL pointers in safe signatures.

- Adds `DmlFrontdoorBaserelView<'a>` and changes the SPIRE DML baserel plan-expression helpers to take that typed view instead of raw `PlannerInfo` / `RelOptInfo` pointers.
- Tightens the constructor into `with_dml_frontdoor_baserel_view(...)`, so the borrowed baserel view is scoped to a closure and cannot be returned with a caller-selected lifetime.
- Adds the reviewer-requested guardrail to `scripts/check_unsafe_comments.sh`: a warning for safe public helper signatures matching `^pub(\(crate\))? fn .*pg_sys::Relation`.

## Validation

- `cargo check --all-targets --no-default-features --features pg18,bench` passed. See `artifacts/cargo-check-pg18-bench.log`.
- `git diff --check b5bc0b50..HEAD` passed. See `artifacts/git-diff-check.log`.
- `bash -n scripts/check_unsafe_comments.sh` passed. See `artifacts/check-unsafe-comments-bash-n.log`.
- The new Relation-signature grep currently reports the existing HNSW helper at `src/am/ec_hnsw/options.rs:299`. See `artifacts/relation-signature-guard.log`.

`bash scripts/check_unsafe_comments.sh` also emits the new warning, then reports existing unsafe-comment baseline drift outside this packet's narrow change. See `artifacts/check-unsafe-comments.log`.

## Reviewer Focus

- Confirm the `with_dml_frontdoor_baserel_view` API adequately prevents the typed baserel view from escaping the planner callback lifetime.
- Confirm the guardrail belongs in `scripts/check_unsafe_comments.sh` / `make audit-unsafe` as the tracked pre-commit-equivalent check.
