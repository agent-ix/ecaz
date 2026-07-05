# Review Request: DML Frontdoor Safe Tree Walkers

## Summary

Code commits:

- `4921b9eb` (`Make SPIRE DML tree walkers safe`)
- `7d3b5a18` (`Clean DML frontdoor walker tests`)

This packet completes the direct Task 50 pass for the rank-4 SPIRE DML frontdoor file. It turns private planner-tree helpers from unsafe caller APIs into safe local APIs after Packet 010 centralized the raw `NodeTag` and tuple-descriptor views. The unsafe boundary now sits inside the helper layer that validates nulls, node tags, and PostgreSQL list ownership before callers consume copied predicate/target-column facts.

Test fixtures that call the newly safe walkers were updated to remove redundant unsafe blocks.

## Unsafe Count

| file | before | after | delta | percent | top-15 target status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_spire/dml_frontdoor/mod.rs` | 146 | 100 | -46 | -31.51% | complete for this slice |

Cumulative Task 50 result for this top-15 file is `160 -> 100`, a `-60` block reduction (`-37.50%`). This satisfies the file's 30% target (`<=112`).

`src/am/ec_spire/dml_frontdoor/tests.rs` has no direct unsafe blocks after the test cleanup.

## Risk / Mitigation

Primary risk is accidentally widening the safe API beyond planner-owned trees. The changed helpers are still private to the DML frontdoor module and retain the same validation: null checks, `NodeTag` gating, and immediate `PgList` consumption. They return owned strings/values or short-lived local decisions; planner pointers do not escape.

No benchmark was run because this packet does not touch scoring, traversal, cache hot paths, or SPIRE read-efficiency loops.

## Validation

- `rustfmt --edition 2021 --check src/am/ec_spire/dml_frontdoor/mod.rs src/am/ec_spire/dml_frontdoor/tests.rs`: passed, with stable-rustfmt warnings for repo config.
- `git diff --check`: passed.
- `cargo check --all-targets --no-default-features --features pg18,bench`: passed with existing unused-import warnings.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`: failed on existing repo-wide lint backlog; see `artifacts/clippy-pg18.log`.
- `cargo test dml_frontdoor --lib --no-default-features --features pg18`: built but failed to launch outside PostgreSQL with `undefined symbol: CacheRegisterRelcacheCallback`; see `artifacts/cargo-test-dml-frontdoor.log`.

Artifacts are under `reviews/task-50/011-dml-frontdoor-safe-tree-walkers/artifacts/`, with artifact metadata in `artifacts/manifest.md`.
