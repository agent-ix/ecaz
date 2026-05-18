# Artifact Manifest: Task 43 Miri Depth Lanes

- Head SHA: `d529e0e8`
- Task bucket: `reviews/task-43/`
- Packet: `reviews/task-43/002-miri-depth-lanes/`
- Timestamp: `2026-05-18T08:56:14-07:00`
- Lane / fixture / storage format / rerank mode: hardening lane wiring; no
  storage fixture; no rerank mode.
- Isolation surface: N/A. This packet validates command wiring only and does
  not run Miri over database-backed storage surfaces.

## `bash-n-hardening.log`

- Command: `script -q -c "bash -n scripts/hardening.sh" reviews/task-43/002-miri-depth-lanes/artifacts/bash-n-hardening.log`
- Key result lines: `COMMAND_EXIT_CODE="0"`

## `make-dry-run-miri-depth.log`

- Command: `script -q -c "make -n miri-tree miri-many-seeds miri-full hardening-nightly-local" reviews/task-43/002-miri-depth-lanes/artifacts/make-dry-run-miri-depth.log`
- Key result lines:
  - `bash scripts/hardening.sh miri-tree`
  - `bash scripts/hardening.sh miri-many-seeds`
  - `bash scripts/hardening.sh miri-full`
  - `COMMAND_EXIT_CODE="0"`

## `make-dry-run-hardening-nightly-local.log`

- Command: `script -q -c "make -n hardening-nightly-local" reviews/task-43/002-miri-depth-lanes/artifacts/make-dry-run-hardening-nightly-local.log`
- Key result lines:
  - `bash scripts/hardening.sh miri-full`
  - `bash scripts/hardening.sh cargo-careful`
  - `COMMAND_EXIT_CODE="0"`

## `hardening-validate.log`

- Command: `script -q -c "bash scripts/hardening_validate.sh" reviews/task-43/002-miri-depth-lanes/artifacts/hardening-validate.log`
- Key result lines: `COMMAND_EXIT_CODE="0"`

## `hardening-tiers-report.log`

- Command: `script -q -c "bash scripts/hardening_tiers_report.sh" reviews/task-43/002-miri-depth-lanes/artifacts/hardening-tiers-report.log`
- Key result lines:
  - `miri-tree               nightly     variable     Tree Borrows Miri prefixes`
  - `miri-many-seeds         nightly     variable     many-seeds Miri prefixes`
  - `COMMAND_EXIT_CODE="0"`
