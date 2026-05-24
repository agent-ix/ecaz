# Task 50 Unsafe Ledger Seed Artifacts

- head SHA: `a2313766d1ba57a836c58903b720afc24ad29299`
- working tree: dirty; includes the paused partial heap-slot helper slice
- task bucket: `reviews/task-50/031-unsafe-ledger-seed/`
- timestamp: `2026-05-20`
- purpose: seed Wave 0 from packet 030 with durable per-unsafe ledger tooling and generated ledger evidence

## Artifacts

- `unsafe-ledger.jsonl`
  - command: `make unsafe-ledger`
  - scope: current direct `unsafe { ... }` blocks under `src/`
  - result: `2446` ledger rows
  - row fields: `id`, `file`, `line_at_capture`, `column_at_capture`,
    `enclosing_item`, `category`, `program`, `disposition`, `status`,
    `residual_reason`, `packet`, `source_excerpt`

- `unsafe-ledger-generate.log`
  - command log for `make unsafe-ledger`

- `unsafe-ledger-check.log`
  - command: `make unsafe-ledger-check`
  - result: `ledger covers 2446 current unsafe rows`

- `unsafe-ledger-program-counts.log`
  - command: program aggregation over `unsafe-ledger.jsonl`
  - result:
    - P1: 28
    - P2: 819
    - P3: 247
    - P4: 300
    - P5: 95
    - P6: 101
    - P7: 48
    - P8: 141
    - P9: 29
    - P10: 70
    - P11: 322
    - P12: 61
    - P13: 185

- `residual-registry.jsonl`
  - initial empty residual registry skeleton
  - remaining unsafe must not be marked complete until represented here with an irreducible reason

## Code Commit

- `53f81340` added `scripts/unsafe_ledger.py` and Make targets.
- `a2313766` completed program classification so current rows are assigned to P1-P13.

