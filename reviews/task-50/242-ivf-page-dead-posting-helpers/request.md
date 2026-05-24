# Task 50 Review Request: IVF Page Dead Posting Helpers

## Summary

Removed unused IVF posting helper surfaces from `src/am/ec_ivf/page.rs`:

- `read_ivf_postings_for_list_blocks_with_tids`
- `visit_ivf_postings_for_block_sequence`
- now-dead `visit_ivf_posting_block_sequence_with_read_stream`
- now-dead pg17 fallback `visit_all_ivf_postings_for_block`
- `rewrite_ivf_posting`

The live ref-based scan path remains intact through
`visit_ivf_posting_refs_for_block_sequence`, and the live vacuum rewrite path
continues to use `rewrite_ivf_postings_for_list_blocks`.

## Unsafe Burndown

- `src/am/ec_ivf/page.rs` unsafe grep count: `48 -> 44`
- repository `src` unsafe grep count: `2482 -> 2478`
- deleted-symbol search returns no remaining references

See `artifacts/unsafe-counts.log`.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_ivf/page.rs`
  - Passed; stable rustfmt emitted the existing unstable-option warnings.
- `git diff --check`
  - Passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - Passed; emitted the existing unused SPIRE re-export warning in
    `src/am/mod.rs`.
- `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run`
  - Passed; emitted the existing Hadamard test-helper dead-code warnings.

## Review Focus

Please verify that the removed helper surfaces were genuinely unused and that
the remaining IVF posting iteration/rewrite entry points still cover the scan
and vacuum paths.
