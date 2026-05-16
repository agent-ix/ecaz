# Artifact Manifest: Task 49 hardening CI governance

Packet: `910-c1-task49-hardening-ci-governance`
Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
Timestamp: `2026-05-16T22:56:55Z`

This packet does not make performance or recall claims. It cites command
pass/fail validation and local macOS behavior for the restored `make test`
target.

## `hardening-validate.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: hardening governance validation
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `bash scripts/hardening_validate.sh > review/910-c1-task49-hardening-ci-governance/artifacts/hardening-validate.log 2>&1`
- Key result lines: command exited 0 with no output.

## `hardening-tiers-report.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: hardening tier inventory
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make hardening-tiers-report > review/910-c1-task49-hardening-ci-governance/artifacts/hardening-tiers-report.log 2>&1`
- Key result lines: `hardening lane tiers at 105ef48f`

## `test-local.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: macOS-safe local unit subset
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make test-local > review/910-c1-task49-hardening-ci-governance/artifacts/test-local.log 2>&1`
- Key result lines: `test result: ok. 331 passed`, `test result: ok. 8 passed`

## `make-test-macos.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: restored full `make test` behavior on local macOS
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make test > review/910-c1-task49-hardening-ci-governance/artifacts/make-test-macos.log 2>&1`, with the nonzero local exit status appended
- Key result lines: `cargo test`, `symbol not found in flat namespace '_BufferBlocks'`, `exit status: 2`

## `fmt-check.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: formatting
- Fixture: current workspace
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `make fmt-check > review/910-c1-task49-hardening-ci-governance/artifacts/fmt-check.log 2>&1`
- Key result lines: command exited 0.

## `git-diff-check.log`

- Head SHA: `105ef48f19a1c56091c4aaa9a076de4b4db01d54`
- Packet/topic: `910-c1-task49-hardening-ci-governance`
- Lane: whitespace
- Fixture: current workspace diff at validation time
- Storage format: text log
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Command used: `git diff --check > review/910-c1-task49-hardening-ci-governance/artifacts/git-diff-check.log`
- Key result lines: command exited 0 with no output.
