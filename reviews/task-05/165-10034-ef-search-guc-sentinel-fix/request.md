# Review Request: ef_search GUC Sentinel Fix

Commit: `bb13a7a`

Scope:
- `src/am/options.rs`

Summary:
- change the session `tqhnsw.ef_search` GUC boot value from `40` to `-1` so the runtime can
  distinguish "unset, use reloption" from "explicitly overridden to 40"
- treat only `-1` as relation fallback inside `resolve_scan_tuning_values(...)`
- keep the effective runtime range at the documented `1..=1000`, with `-1` reserved for the
  boot/default sentinel
- extend unit coverage to prove that `SET tqhnsw.ef_search = 40` wins over a different index
  reloption instead of being silently dropped

Please review:
- whether reserving `-1` as the unset sentinel is the cleanest long-lived contract for the session
  GUC
- whether the updated test coverage is sufficient for the explicit-default override case
- whether any downstream planner/debug wording should be tightened now that the control surface no
  longer conflates "unset" with "40"
