# Task 43 Review Request: Mutation Probes

## Scope

This packet closes G7 in the Task 43 campaign tracker. It proves the expanded
Miri tests are not just presence checks by temporarily injecting obvious bugs
into each major subsystem and recording the targeted failure.

Subsystems covered:

- Common parallel shared state.
- DiskANN graph pruning.
- HNSW stale-frontier filtering.
- DiskANN vacuum fully-dead semantics.
- SPIRE top-k / candidate merge tie ordering.
- SPIRE routing adaptive nprobe reduction.
- SPIRE vacuum/delete-delta visibility.
- Remote typed payload byte-cap validation.
- SPIRE serialization duplicate vec-id rejection.

No mutation remains in source. Each patch was saved under
`artifacts/patches/`, the failing Miri log was saved under `artifacts/`, and
the source file was restored before committing.

## Evidence

- `artifacts/manifest.md` contains the full mutation matrix with commands and
  expected failures.
- All nine mutation logs record `test result: FAILED` and
  `COMMAND_EXIT_CODE="101"`.
- All nine patch files are packet-local under `artifacts/patches/`.
- `artifacts/source-status-after-probes.log` records no remaining changes under
  `src`, `hardening`, or `docs`.
- `artifacts/git-diff-check.log` and `artifacts/cargo-fmt-check.log` both exit
  0.

## Review Focus

- Check that each major subsystem named by the tracker has a real mutation
  probe.
- Check that each probe mutates production code, not only test assertions.
- Check that the failures are meaningful for the intended safety contract.
- Check that the committed tree contains no remaining mutation.
