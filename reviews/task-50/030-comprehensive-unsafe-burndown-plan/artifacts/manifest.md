# Task 50 Comprehensive Unsafe Burndown Plan Artifacts

- head SHA: `4018bd29`
- working tree: dirty; includes an uncommitted partial heap-slot helper slice
- task bucket: `reviews/task-50/030-comprehensive-unsafe-burndown-plan/`
- timestamp: `2026-05-20`
- purpose: replace the narrow Task 50 execution ladder with a complete strategic burndown plan covering every direct unsafe block
- plan source: `../request.md`

## Artifacts

- `src-unsafe-block-count-working-tree.log`
  - command: `make unsafe-block-count`
  - scope: direct `unsafe { ... }` blocks under `src/`
  - result: `2446` direct unsafe blocks across `131` files in the current working tree

- `src-unsafe-block-lines-working-tree.log`
  - command: `rg -n "unsafe\s*\{" src --glob '*.rs'`
  - scope: line-level direct unsafe inventory under `src/`
  - result: one row per direct unsafe block; this is the line-level ledger seed

- `repo-unsafe-block-count-working-tree.log`
  - command: `rg -n "unsafe\s*\{" --glob '*.rs' --glob '!target/**' --glob '!reviews/**' --glob '!review/**' --glob '!benchmarks/**' . | cut -d: -f1 | sort | uniq -c | sort -nr`
  - scope: owned and checked-in Rust outside review artifacts, including `src/`, `hardening/`, `crates/`, and `vendor/`
  - result: identifies non-`src` unsafe in hardening, small crates, and vendored code for separate disposition

- `subsystem-totals-working-tree.log`
  - command: subsystem aggregation over `make unsafe-block-count`
  - result:
    - HNSW: `797` blocks / `12` files
    - SPIRE: `771` blocks / `45` files
    - IVF: `252` blocks / `10` files
    - DiskANN: `195` blocks / `7` files
    - tests: `185` blocks / `36` files
    - AM common: `84` blocks / `7` files
    - root/other: `57` blocks / `4` files
    - quant: `55` blocks / `2` files
    - storage guards: `50` blocks / `8` files

- `pattern-candidate-counts-working-tree.log`
  - command: `rg` pattern inventory over `src/**/*.rs`
  - result: candidate unsafe families by recurring operation, used to choose contracts before touching individual files
