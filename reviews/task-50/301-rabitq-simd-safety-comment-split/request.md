# Review Request: RaBitQ SIMD Safety Comment Split

## Summary

This checkpoint responds to the soundness audit finding in
`reviews/task-50/132-helper-soundness-audit/feedback/2026-05-21-01-reviewer.md`.

The audit flagged two RaBitQ AVX2/FMA SAFETY comments in `src/quant/prod.rs`
as combining multiple invariants in one comment, which made future audits
harder. This patch splits those comments into one invariant per SAFETY line.
While doing that, it applies the same split to the matching NEON checked paths
so the SIMD guard documentation stays coherent across architectures.

This is documentation-only and intentionally does not change behavior.

## Code Commit

- `54ae34331bcac6f93c7d3b4ffb3040e26e8ccc48` - `Split RaBitQ SIMD safety invariants`

## Unsafe Count

- Previous packet baseline after packet 300: `2061`
- After this checkpoint: `2061`
- Net change: `0`
- `src/quant/prod.rs` by-file match count remains `13`

## Ledger

- Generated packet-local ledger: `artifacts/unsafe-ledger-after.jsonl`
- `unsafe-ledger-check.log`: `ledger covers 1389 current unsafe rows`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe src | wc -l`
- `make UNSAFE_LEDGER=reviews/task-50/301-rabitq-simd-safety-comment-split/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/301-rabitq-simd-safety-comment-split unsafe-ledger`
- `make UNSAFE_LEDGER=reviews/task-50/301-rabitq-simd-safety-comment-split/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`

`cargo check` passes. The log includes the known pre-existing SPIRE unused-import
warning in `src/am/mod.rs`.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/unsafe-line-count.log`
- `artifacts/unsafe-ledger-after.jsonl`
- `artifacts/unsafe-ledger-generate.log`
- `artifacts/unsafe-ledger-check.log`
