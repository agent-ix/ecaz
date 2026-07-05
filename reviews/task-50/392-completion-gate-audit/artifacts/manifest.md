# Manifest: Completion Gate Audit

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/392-completion-gate-audit`
- Branch: `task-50-unsafe-closeout`
- Head at audit start: `1f6046130280dc971c948b655fb7fb6ff66e350c`
- Purpose: verify whether the completion claim satisfies the closeout gate in
  `reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`
- Result: not complete

## Artifacts

- `plan-closeout-source-a.log`
  - Command: `sed -n '1,160p' reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`
  - Purpose: source objective and strategic-method requirements.

- `plan-closeout-source-b.log`
  - Command: `sed -n '380,520p' reviews/task-50/030-comprehensive-unsafe-burndown-plan/request.md`
  - Purpose: source execution order, packet acceptance rules, and closeout gate.

- `src-unsafe-count-current.log`
  - Command: `rg -n 'unsafe\\s*\\{' src | wc -l`
  - Key result: `1124`

- `repo-scope-unsafe-count-current.log`
  - Command: `rg -n 'unsafe\\s*\\{' src hardening crates vendor --glob '*.rs' | wc -l`
  - Key result: `1252`

- `non-src-unsafe-count-current.log`
  - Command: `rg -n 'unsafe\\s*\\{' hardening crates vendor --glob '*.rs' | wc -l`
  - Key result: `128`

- `src-unsafe-count-by-file-current.log`
  - Command: `rg -n 'unsafe\\s*\\{' src --glob '*.rs' | cut -d: -f1 | sort | uniq -c | sort -nr`
  - Purpose: current per-file direct unsafe count.

- `non-src-unsafe-count-by-file-current.log`
  - Command: `rg -n 'unsafe\\s*\\{' hardening crates vendor --glob '*.rs' | cut -d: -f1 | sort | uniq -c | sort -nr`
  - Purpose: current non-`src` count by file.

- `subsystem-totals-current.log`
  - Command: grouped `rg`/`awk` count over current `src` direct unsafe rows.
  - Key result:
    - `549 9 HNSW`
    - `199 34 SPIRE`
    - `93 12 AM common`
    - `78 13 Storage guards`
    - `65 7 DiskANN`
    - `61 7 IVF`
    - `33 7 Tests`
    - `32 2 Quant`
    - `14 3 Root / other`

- `current-unsafe-ledger.jsonl`
  - Command: `python3 scripts/unsafe_ledger.py generate --output reviews/task-50/392-completion-gate-audit/artifacts/current-unsafe-ledger.jsonl --packet reviews/task-50/392-completion-gate-audit src`
  - Key result: `1124` current ledger rows.

- `current-unsafe-ledger-generate.log`
  - Command log for current ledger generation.

- `current-unsafe-ledger-check.log`
  - Command: `python3 scripts/unsafe_ledger.py check --ledger reviews/task-50/392-completion-gate-audit/artifacts/current-unsafe-ledger.jsonl src`
  - Key result: `ledger covers 1124 current unsafe rows`.

- `current-ledger-open-status-count.log`
  - Command: `rg '"status": "open"' reviews/task-50/392-completion-gate-audit/artifacts/current-unsafe-ledger.jsonl | wc -l`
  - Key result: `1124`.

- `current-ledger-sample.log`
  - Command: `sed -n '1,5p' reviews/task-50/392-completion-gate-audit/artifacts/current-unsafe-ledger.jsonl`
  - Purpose: sample rows showing `status=open`, `disposition=planned`.

- `residual-artifact-files.log`
  - Command: `find reviews/task-50 -path '*/artifacts/*residual*' -type f | sort`
  - Key result: the only true residual registry artifact found is
    `reviews/task-50/031-unsafe-ledger-seed/artifacts/residual-registry.jsonl`.

- `residual-and-ledger-wc.log`
  - Command: `wc -l reviews/task-50/031-unsafe-ledger-seed/artifacts/residual-registry.jsonl reviews/task-50/392-completion-gate-audit/artifacts/current-unsafe-ledger.jsonl`
  - Key result: residual registry has `0` rows; current audit ledger has `1124`.

- `latest-ledger-artifacts.log`
  - Command: `find reviews/task-50 -path '*/artifacts/unsafe-ledger-after.jsonl' -type f | sort | tail -5`
  - Purpose: identify latest implementation ledger artifacts before this audit.

- `closeout-like-packets.log`
  - Command: `find reviews/task-50 -maxdepth 2 -type f -name request.md | sort | rg 'closeout|final|residual|completion|gate|ledger-refresh'`
  - Purpose: inspect whether a final closeout/residual packet exists.

- `latest-feedback-files.log`
  - Command: `find reviews/task-50 -path '*/feedback/*.md' -type f | sort | tail -20`
  - Purpose: recent reviewer feedback inventory.

- `recent-feedback-summary.log`
  - Command: `rg -n 'approve|request changes|bug|unsound|incorrect|block|fail' reviews/task-50/38*-*/feedback reviews/task-50/39*-*/feedback -S`
  - Purpose: recent feedback signal; recent packets through 391 are approvals,
    but approval of slices is not equivalent to final closeout.

- `raw-boundary-guard-current.log`
  - Command: `rg -n '^pub(\\(crate\\))? fn .*pg_sys::(Relation|IndexScanDesc|StringInfo|ParamListInfo|Query|PlannerInfo|RelOptInfo|Node|Expr|List|TupleTableSlot|ScanKey|IndexBuildHeapScan|IndexVacuumInfo|IndexBulkDeleteResult)' src`
  - Result: no matches; command exit code `1` from `rg` means empty result.
