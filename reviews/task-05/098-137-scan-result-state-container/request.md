# Request: Group Scan Result State In One Container

Commit: `27e8545`

Summary:
- replace the parallel `current_result` plus pending duplicate-drain fields on `TqScanOpaque` with one explicit `ScanResultState`
- move result-state operations behind that owned container:
  - clear pending heap tids
  - store pending heap tids
  - take next pending heap tid
  - set current result
  - update current-result heap progress
- update scan debug/test surfaces to read current-result and pending-drain state through the new container

Please review:
- whether `ScanResultState` is the right structural boundary for the current staged executor
- whether any scan/debug path still leaks old assumptions about result and duplicate-drain state being separate
- whether this leaves a cleaner foundation for a later search-owned result cursor or ordered traversal state
