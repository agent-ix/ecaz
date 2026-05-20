# Task 50 Packet 013: IVF Scan Storage and Debug Facade

## Code Under Review

- Code commit: `137ff87f18fe8b5790511c243df6cc4eabd25a76`
- File touched: `src/am/ec_ivf/scan.rs`
- Task: `plan/tasks/50-unsafe-structural-reduction.md`

## Scope

This packet completes the top-15 Task 50 pass for `src/am/ec_ivf/scan.rs`.
It reduces repeated unsafe blocks in two surfaces:

- scan-local `palloc` slice storage for query values, centroid scores, selected
  lists, and posting candidates;
- test/debug scan helpers for AM begin/rescan/gettuple/end, metadata reads,
  scan opaque access, heap TID reads, and order-by score inspection.

The production allocation/copy/free behavior is unchanged: each scan-local
slice is still copied into PostgreSQL memory and released when the scan opaque
is reset or ended.

## Unsafe Block Count

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_ivf/scan.rs` | 102 | 69 | -33 | -32.4% | Meets >=30% target |

No benchmark lane is claimed. The production change centralizes allocation and
copy wrappers without changing scoring, traversal order, probe selection, or
cache policy; the larger reduction is in debug/test scan accessors.

## Validation

- `make unsafe-block-count PATHS='src/am/ec_ivf/scan.rs'`
  - before: `102 src/am/ec_ivf/scan.rs`
  - after: `69 src/am/ec_ivf/scan.rs`
- `rustfmt --edition 2021 --check src/am/ec_ivf/scan.rs`: passed with existing
  stable-rustfmt warnings about unstable config keys.
- `cargo check --all-targets --no-default-features --features pg18,bench`:
  passed with existing warnings outside this slice.
- `git diff --check`: passed.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`:
  still blocked by the existing repo-wide clippy backlog; first failures remain
  outside this slice.
- `cargo fmt --all --check`: still reports pre-existing formatting drift in
  files outside this slice; touched-file rustfmt check passed.
- `cargo test ec_ivf --lib --no-default-features --features pg18`: built the
  test binary, then failed to launch outside PostgreSQL with
  `undefined symbol: CacheRegisterRelcacheCallback`.

## Artifacts

- `artifacts/block-count-before.log`
- `artifacts/block-count-after.log`
- `artifacts/rustfmt-touched-check.log`
- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-clippy-pg18.log`
- `artifacts/cargo-test-ec-ivf.log`
- `artifacts/git-diff-check.log`
- `artifacts/manifest.md`
