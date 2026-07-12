---
task: 179
packet: 039-remote-endpoint-security
role: coder
status: review-requested
head: 1c1490107d87772456fcdf16269cc004d432bef7
date: 2026-07-12
---

# Review request: close the remote endpoint privilege class

Please review commit `1c1490107`, the exact-SHA PG18 evidence under
`artifacts/`, and the scoped decision in `verdict.md`.

This checkpoint responds to packet 033 P1-1 and P2-1. The requested decisions
are:

1. Are both named M2 siblings—the oid-signature `ec_distann_expand_nodes` and
   `ec_distann_materialize_rows`—now SECURITY DEFINER with a fixed safe search
   path and no PUBLIC execute privilege?
2. Does the class-level regression test prevent recurrence across all eight
   current expansion, materialization, and remote-write overloads rather than
   checking only individually named wrappers?
3. Is the legacy remote write endpoint equivalently secured, and does it reject
   stronger isolation before relation access while preserving normal READ
   COMMITTED tombstoning?
4. Do the FR-079/FR-083 updates make those class-wide privilege and isolation
   requirements normative?

Exact-SHA warnings-denied PG18 clippy passes. The installed-extension ACL test
audits all eight current overloads and passes. The combined write test passes
both the invalid-OID Repeatable Read rejection and the existing normal
tombstone-success case.

This request is scoped to packet 033 P1-1 and P2-1. Packet 033 P1-2 (the
Cancelled-decision orphan reclaim/spec mismatch), its carried P2s, Task 179
benchmark closeout work, and outside review of packets 035–038 remain open.
