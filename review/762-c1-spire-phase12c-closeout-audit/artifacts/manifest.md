# Artifact Manifest: SPIRE Phase 12c Closeout Audit

- head SHA: `aac40104fea270765672e163ef3bddaaa0ab559b`
- packet/topic: `762-c1-spire-phase12c-closeout-audit`
- lane: Phase 12c test coverage closeout
- fixture: not applicable; closeout audit packet
- storage format: not applicable
- rerank mode: not applicable
- command surface: tracker, file-size, and remote-ref verification
- timestamp: `2026-05-15T02:28:44Z`
- isolated one-index-per-table vs shared-table surface: not applicable

## Commands

- `git status --short --branch`
- `git rev-parse HEAD`
- `git ls-remote origin refs/heads/task-30-spire`
- `rg -n "^- \\[ \\]" plan/tasks/task30-phase12c-spire-test-coverage.md`
- `find src/tests src/am/ec_spire -type f -name '*.rs' -exec wc -l {} + | sort -nr | head -25`
- `git diff --name-only cfa7e89e..HEAD -- 'src/am/ec_spire/**/*.rs' 'src/am/ec_spire/*.rs' | sort`
- `rg -n "for_test|cfg\\(any\\(test|cfg\\(test|pg_test" src/am/ec_spire/custom_scan/mod.rs src/am/ec_spire/custom_scan/begin_exec.rs src/am/ec_spire/mod.rs src/am/ec_spire/coordinator/remote_candidates/resolve.rs src/am/ec_spire/coordinator/remote_candidates/write_payload.rs`

## Key Result Lines

- Local `HEAD`: `aac40104fea270765672e163ef3bddaaa0ab559b`
- Remote `task-30-spire`: `aac40104fea270765672e163ef3bddaaa0ab559b`
- Phase 12c unchecked-row search returned no matches.
- Largest SPIRE-side files touched by Phase 12c after split:
  - `2492 src/tests/mod.rs`
  - `2404 src/tests/dml_frontdoor.rs`
  - `2317 src/tests/insert.rs`
  - `1758 src/tests/remote_search/epoch_manifest.rs`
  - `1657 src/tests/remote_search/contracts.rs`
  - `1652 src/tests/remote_search/receive_faults.rs`
  - `1208 src/tests/remote_search/contracts_libpq.rs`
- Remaining >2500-line test files shown by the broad audit are HNSW files:
  `ec_hnsw_scan_gettuple.rs`, `ec_hnsw_recall_debug_exports.rs`,
  `ec_hnsw_runtime_profiles.rs`, and `ec_hnsw_storage_lifecycle.rs`.
- Non-test SPIRE production paths changed for testability are concentrated in:
  `custom_scan/{mod.rs,begin_exec.rs}`,
  `coordinator/remote_candidates/{resolve.rs,write_payload.rs}`, and
  `mod.rs`; the observable assertions are gated through `#[cfg(test)]` /
  `#[cfg(any(test, feature = "pg_test"))]` hooks.
