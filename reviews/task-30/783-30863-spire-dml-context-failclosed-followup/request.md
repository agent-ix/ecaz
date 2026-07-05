# Review Request: SPIRE DML Context Fail-Closed Follow-Up

## Scope

This packet addresses the P2 follow-ups from the 30861 fail-closed guard
review. It does not change the DML plan-rewrite state.

Code commit: `e78724fc5a18d6bd4f4af826fb491b20cf265c21`

Changes:

- Adds a PG18 fixture for the `relation_context_error` planner-hook path by
  creating a table with two `ec_spire` indexes and running an actual PK SELECT.
- Asserts the planner hook surfaces the context-load failure message and
  ADR-069 hint verbatim.
- Asserts hook diagnostics record `last_hook_action =
  planner_error_fail_closed` and `last_classification_kind =
  relation_context_error`.
- Documents that hook diagnostics are intentionally backend-local.
- Documents that the fail-closed guard intentionally runs before chained
  planner-hook delegation so unsupported distributed DML cannot be rewritten
  into a coordinator-heap plan by another extension.

## Validation

- `cargo test dml_frontdoor --lib`
  - 19 passed, 0 failed, 1648 filtered out.
- `cargo fmt --check`
  - Passed with the existing stable-rustfmt warnings about unstable import
    options.
- `git diff --check`
  - Passed.

Artifacts are recorded in `artifacts/manifest.md`.

## Review Focus

1. Confirm the new fixture covers the context-error fail-closed path requested
   in the 30861 review.
2. Confirm the comments accurately describe backend-local diagnostics and
   pre-delegation guard ordering.
3. Confirm no plan-rewrite behavior changed in this follow-up.
