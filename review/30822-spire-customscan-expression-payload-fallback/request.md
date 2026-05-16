# Review Request: SPIRE CustomScan Expression Payload Fallback

## Scope

Feedback follow-up for the P2 in
`review/30815-spire-customscan-loopback-read/feedback/2026-05-11-001-reviewer.md`.

The bug: CustomScan tuple-payload column narrowing only collected bare
projected `Var`s. A target list like `SELECT id, title || ' suffix' ...`
requested `id` but omitted `title`, so the expression could evaluate against a
NULL payload slot.

This slice:

- Treats projection narrowing as valid only when every non-junk target entry is
  a simple user-column `Var`.
- Falls back to requesting the full relation payload when any projected target
  entry is an expression or system-column Var.
- Extends the existing loopback CustomScan fixture with:
  `SELECT id, title || ' (boosted)' ... ORDER BY embedding <#> ... LIMIT 1`.
- Asserts the expression result is `remote alpha (boosted)`, proving the remote
  payload contains the expression input column.

This chooses the conservative fallback recommended by the reviewer rather than
adding a full expression-tree Var walker in this packet. That keeps expression
projections correct before later typed tuple transport or remote projection
pushdown work.

## Validation

- `cargo test customscan_returns_loopback_remote_tuple_payload --lib`
  - Passed: 1 test.
- `cargo fmt --check`
  - Passed with the repository's existing stable-rustfmt warnings about
    nightly-only import options.
- `git diff --check`
  - Passed.
- `git diff --cached --check`
  - Passed before the code commit.

## Review Focus

- Check the fallback boundary: mixed simple Var plus expression target lists now
  request all relation columns instead of an incomplete subset.
- Confirm the loopback assertion covers the previous wrong-results shape.
- Confirm this does not claim final expression-level projection pushdown; it is
  a correctness fallback.

## Artifacts

- `review/30822-spire-customscan-expression-payload-fallback/artifacts/manifest.md`
- `review/30822-spire-customscan-expression-payload-fallback/artifacts/cargo-test-customscan-loopback-expression-lib.log`
- `review/30822-spire-customscan-expression-payload-fallback/artifacts/cargo-fmt-check.log`
- `review/30822-spire-customscan-expression-payload-fallback/artifacts/git-diff-check.log`
- `review/30822-spire-customscan-expression-payload-fallback/artifacts/git-diff-cached-check.log`
