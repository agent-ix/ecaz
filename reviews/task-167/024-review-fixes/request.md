# Task 167 review fixes

Please review code checkpoint `12ab7c44e`.

This checkpoint addresses the latest findings by:

- restoring strict typed `OwnedRecordMissing` behavior for physical frontier
  lookup, with one fresh-snapshot retry for a transient post-commit visibility
  window;
- retaining the pinned owner probe and requiring an exact owner-side sampled-row
  probe, while logging coordinator equality-probe zero rows explicitly;
- distinguishing missing row-tier tuples from not-visible tuples during backlink
  maintenance;
- making free-capacity backlink amendments append-preserving and fixing the
  full-target amendment code-copy source;
- replacing the old vacuous concurrency check with a seeded shared-target drill
  that resolves stable vec IDs and requires both controlled writer edges.

PG18 `cargo check --features pg18` and CLI compile checks pass. The physical
fixture was exercised during development, but the installed extension
preflight reported the prior extension SHA (`0a7854fc...`) rather than this
checkpoint, so no exact-head runtime pass or benchmark claim is made here.
The packet remains review-open and Task 167 still requires the mandated
10k/50k/100k benchmark evidence plus outside ACCEPT.

head_sha: 12ab7c44e
task_bucket: reviews/task-167
packet: 024-review-fixes
timestamp: 2026-08-13 (America/Los_Angeles)
