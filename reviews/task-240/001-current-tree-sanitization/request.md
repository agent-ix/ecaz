---
task: 240
packet: 001-current-tree-sanitization
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 240 current-tree sanitization review request

Please review the current-tree privacy repair and future-publication guard at
`e393515c7c94e0f127bcbac633052ee457c0527e`, based on public main
`23fb9b7ba1f0803be5dfc700d9865f80fbf60862`.

## Requested decision

Accept the implementation if the reviewer agrees that:

1. the 31 current-tree references to the identified internal scratch locator
   were replaced with durable public citations, source identities, explicit
   operator inputs, or a neutral local-checkout marker;
2. those edits changed locator/citation metadata only and did not alter
   benchmark measurements, source SHAs, findings, or technical conclusions;
3. the CLI must require `--repo` or the documented environment variable for
   both external comparison-extension installers;
4. the added-lines guard safely reports file, line, and category without
   echoing the matched value; and
5. Git-history remediation remains explicitly out of scope until a separate
   destructive-history decision is approved.

## Implementation

- Removed the identified internal scratch locator from every tracked file in
  the current tree.
- Replaced paper locations with arXiv identifiers and source checkouts with
  durable repository identities plus pinned SHAs where already available.
- Removed machine-specific comparison-repository defaults from
  `ecaz dev install pgvector` and `ecaz dev install vectorscale`.
- Added `scripts/check_workstation_paths.py`, focused synthetic tests, and a
  lightweight pull-request/main-push workflow.
- Added the workstation-path rule to `AGENTS.md`.

The guard covers added Unix, root, Windows, and tilde-home paths. It inspects
only added diff lines so pre-existing immutable evidence does not cause every
unrelated pull request to fail.

## Deliberate boundary

The baseline contains 6,396 tracked files with one common absolute workstation
prefix. This checkpoint reduces that exact-prefix count to 6,390 but does not
bulk-rewrite the legacy evidence corpus. It does reduce the specifically
identified internal scratch locator from 31 tracked files to zero.

Old values remain reachable from existing Git objects. No branch, tag, pull
request ref, commit, or repository history was rewritten or deleted. A
history-wide purge would require a coordinated force-push, collaborator
re-clone procedure, and hosting-provider cache/ref handling; that is not
authorized by this packet.

## Validation

- Focused Python tests: 3/3 pass.
- Added-lines guard over `23fb9b7...e393515c7`: pass.
- Current-tree exact-locator audit: zero matches.
- `git diff --check`: pass.
- `cargo fmt --all -- --check`: pass (existing stable-rustfmt warnings only).
- Isolated `cargo build -p ecaz-cli`: pass (one existing unread-field warning).
- `cargo test -p ecaz-cli commands::dev::install`: compile pass; the filter
  selected zero tests, so this is not claimed as behavioral coverage.
- New binary, repository variables unset:
  - pgvector installer: exit 2, missing required `--repo <REPO>`;
  - vectorscale installer: exit 2, missing required `--repo <REPO>`.

The repository-wide `scripts/tests/run.sh` is not green on the unchanged
baseline: it reports three failures and four errors in the socket-resolution
tests because two referenced shell scripts are already absent. This task does
not modify that unrelated surface and does not claim the full runner as a pass.

See `artifacts/manifest.md` and `artifacts/redaction-audit.md` for the exact
scope and evidence classification.
