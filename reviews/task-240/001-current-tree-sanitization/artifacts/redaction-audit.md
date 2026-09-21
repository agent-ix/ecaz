# Task 240 current-tree redaction audit

## Classification

The 31 baseline files containing the identified internal scratch locator were:

| Category | Files | Treatment |
|---|---:|---|
| CLI source and documentation | 2 | Removed implicit local defaults; documented explicit operator inputs |
| Benchmark/reviewer evidence | 2 | Replaced only source/build locator text |
| Planning and design | 4 | Replaced local locations with durable public identities |
| Review packets | 18 | Replaced citation/source locators; retained findings and measurements |
| Specifications, ADRs, and findings | 5 | Replaced local citations and closed the resolved citation finding |
| **Total** | **31** | **Current-tree locator count is zero** |

## Evidence-integrity check

The diff was reviewed line by line. Existing evidence edits are confined to:

- paper-location to arXiv-identifier substitutions;
- checkout-location to public repository/source-identity substitutions;
- local build-path components to a neutral `[local-checkout]` marker; and
- prose wrapping needed by those substitutions.

No measured value, result row, duration, source SHA, algorithm description,
review verdict, or acceptance conclusion changed. The edited historical build
log is described by its manifest but is not content-digest-bound there.

## Residual exposure

This is a current-tree repair, not a history purge. Existing Git objects retain
the old bytes, and the legacy corpus still contains other workstation paths.
The new guard prevents newly added instances; it does not retroactively reject
or silently rewrite pre-control evidence.

Any later history remediation must have its own authorization, recovery plan,
force-push coordination, hidden-ref/cache verification, and post-rewrite audit.
