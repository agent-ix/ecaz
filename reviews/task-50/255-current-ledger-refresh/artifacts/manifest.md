# Artifact Manifest

- head SHA: `fa84d46190308f58c4958033b7848e4a159be15c`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/255-current-ledger-refresh`
- timestamp: `2026-05-21T15:22:30Z`
- lane: Wave 5 ledger refresh / current unsafe inventory
- fixture: local ledger/static validation
- storage format: not applicable
- rerank mode: not applicable
- table/index isolation: not applicable

## Artifacts

- `artifacts/unsafe-ledger-after.jsonl`
  - Command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/255-current-ledger-refresh/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/255-current-ledger-refresh`
  - Result: generated `1640` current direct unsafe ledger rows under `src`.
- `artifacts/unsafe-ledger-generate.log`
  - Command log for the ledger generation.
- `artifacts/unsafe-ledger-check.log`
  - Command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/255-current-ledger-refresh/artifacts/unsafe-ledger-after.jsonl`
  - Result: `ledger covers 1640 current unsafe rows`.
- `artifacts/unsafe-ledger-program-counts.log`
  - Command: aggregated refreshed ledger rows by `program`.
  - Key lines include `P2 547`, `P4 234`, `P13 193`, `P11 157`, and `P8 130`.
- `artifacts/current-unsafe-counts.log`
  - Command: counted current direct unsafe blocks, broad unsafe grep hits,
    files with direct unsafe blocks, and top direct-unsafe files.
  - Key lines:
    - `current direct unsafe blocks: 1640`
    - `current unsafe grep hits: 2405`
    - `current files with direct unsafe blocks: 132`
