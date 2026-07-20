---
task: 179
packet: 064-lifecycle-authority-decomposition
role: coder
status: review-requested
head: 0043c3e746bef0baf6977dc8ae426006d7a0a887
date: 2026-07-13
---

# Review request: lifecycle authority and phase decomposition

Please review code commit `0043c3e74` as the P2-8/P2-9 and named P3
remediation from packet 060.

This checkpoint:

- defines typed registration, publish-decision, generation, and predecessor-
  disposition state authorities with one legal-transition checker;
- routes persisted-state decoding and every changed-row transition through
  those types, including the formerly unchecked Registered-to-Ready update;
- physically decomposes the coordinator into T1, T2, T3, T4a, and cancellation
  implementation modules (T4b and abandonment remain in their existing
  dedicated modules), leaving only shared lock/catalog/status support in the
  parent;
- preserves underlying SPI error detail at every reviewer-named coordinator
  lookup/insert site;
- splits the former 9,078-line basic test include into a 2,590-line basic file,
  a physical lifecycle include, and a scan-registry include; and
- replaces the permanently ignored preload test with a real two-backend PG18
  shared-memory/fence contention test.

The registry test uses the production shared registration fence rather than a
test-only lock approximation. Full commands, hashes, and PASS lines are in
`artifacts/manifest.md`.

Review focus:

- completeness and legality of the four transition graphs;
- phase-module ownership and absence of duplicated SQL endpoints;
- transaction/count checks at state changes;
- cross-backend fence/token semantics; and
- whether the lifecycle-area test split is maintainable.
