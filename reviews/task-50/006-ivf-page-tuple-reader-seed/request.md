# Task 50 Packet 006: IVF Page Tuple Reader Seed

## Code Under Review

- Commit: `6bdf5335426d74fd3a8de36fd43eaec3e5878eba`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`
- Slice: 2, IVF/RaBitQ page tuple visitor seed.

## Scope

This packet introduces a lifetime-scoped `PageTupleReader` in
`src/am/ec_ivf/page.rs`. The reader is constructed from a `LockedBufferGuard`,
caches the page pointer, page size, block number, and line-pointer count, and
centralizes the proof that tuple bytes are exposed only while the buffer guard
is live.

The first rollout covers read-only tuple access:

- required tuple reads in `read_page_tuple`;
- forward tag scans in `find_next_tuple_with_tag`;
- posting tuple iteration from locked buffers;
- borrowed posting tuple ref iteration from locked buffers;
- debug posting-block summary line-pointer iteration.

The read-only posting buffer visitors are now safe functions because the
`LockedBufferGuard` argument proves the page is pinned and locked, while
`PageTupleReader` validates item-id bounds before exposing tuple bytes.

This is deliberately a seed packet. It does not change WAL mutation helpers,
exclusive-buffer rewrite logic, posting append behavior, directory rewrites, or
metadata page special-area access. Those remain the next IVF page-reader
rollout surfaces needed to move `src/am/ec_ivf/page.rs` toward the Task 50
30% top-15 target.

## Unsafe Block Count

```text
file | before | after | delta | percent | top-15 target status
src/am/ec_ivf/page.rs | 134 | 122 | -12 | -9.0% | in progress; 30% target requires follow-on mutable/WAL page helpers
```

## Risk Register

Relevant row: `IVF page tuple visitor`.

- Failure mode: tuple bounds, item-id interpretation, or error ordering drift.
- Mitigation: keep the existing `with_page_line_tuple_bytes` validation helper
  as the single raw-byte exposure point; make `PageTupleReader` a small wrapper
  over the existing helper; keep mutation paths out of this seed packet.
- Verification: compile check, touched-file formatting, direct block-count
  delta, and focused page test attempt recorded below.

## Validation

- PASS: `cargo check --all-targets --no-default-features --features pg18,bench`
  - Log: `artifacts/cargo-check-pg18-bench.log`
- PASS: touched-file `rustfmt --check src/am/ec_ivf/page.rs`
  - Log: `artifacts/rustfmt-touched-check.log`
- PASS: `git diff --check`
  - Log: `artifacts/git-diff-check.log`
- FAIL, host runtime limitation:
  `cargo test --lib --no-default-features --features pg18,bench am::ec_ivf::page:: -- --nocapture`
  - Log: `artifacts/cargo-test-ivf-page-pg18.log`
  - The test binary builds, then fails to execute outside PostgreSQL with
    `undefined symbol: LockBuffer`.
- FAIL, pre-existing repo drift: `cargo fmt --all --check`
  - Log: `artifacts/cargo-fmt-check.log`
  - Existing formatting diffs remain in `hardening/careful/src/spire_diagnostics_helpers.rs`
    and `src/quant/simd.rs`.
- FAIL, pre-existing repo-wide clippy backlog:
  `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  - Log: `artifacts/cargo-clippy-pg18.log`
  - The final clippy log has no diagnostics for `src/am/ec_ivf/page.rs`.

No benchmark was run. This seed changes the read-only visitor shape but does
not change IVF/RaBitQ scoring, posting append, rewrite, WAL, or page compaction
logic.
