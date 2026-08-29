# Task 240: Artifact Path Sanitization

Status: **implementation in progress** (2026-08-29). Priority: P0 privacy and
evidence hygiene. Origin: private Engineering Assurance gap audit.

## Why

The public default branch contains machine-specific workstation paths in authored
configuration and historical evidence. GitHub code search exposed only a subset; a
local tracked-file audit found 6,396 affected files. The values disclose local layout,
are non-portable, and can accidentally identify rights-unreviewed source locations.

## Goal

Remove the explicitly identified internal scratch locator from the current tree, make
external-source references durable, and prevent any newly added workstation path from
reaching GitHub without rewriting historical benchmark evidence blindly.

## Scope

1. Replace current authored internal-source locators with durable public identifiers or
   explicit operator inputs.
2. Remove machine-specific default comparison-repository locations from `ecaz dev
   install`; require `--repo` or the documented environment variable.
3. Add a diff-aware guard that detects Unix, root, and Windows workstation home paths,
   reports only file/line/category, and never repeats the matched value in CI logs.
4. Run the guard on every pull request and main-branch push in a lightweight workflow.
5. Preserve a transparent distinction between current-tree redaction and historical Git
   reachability. Do not rewrite history or silently change digest-bound evidence.

## Acceptance

1. The current tree contains no reference to the internal scratch locator identified by
   the audit.
2. Synthetic tests prove added Unix, root, and Windows workstation paths fail while
   removals and portable environment-based paths pass.
3. Failure output contains the file, line, and category but not the sensitive value.
4. The CLI comparison installers have no machine-specific repository default.
5. The lightweight GitHub workflow enforces the guard without invoking benchmark or
   PostgreSQL lanes.
6. A review packet records the legacy-corpus count, redaction boundary, test evidence,
   and the separate unresolved history-remediation decision.
