---
task: 227
packet: 002-query-slicing
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 227 exact staged-query slicing

This packet requests review of code checkpoint `231853f18`. It adds the first
diagnostic-tooling prerequisite: `query_offset` on the suite-driven DistANN
multinode step and its dev fixture.

For real staged corpora, the fixture now loads exactly `queries` rows after the
zero-based offset into `dm_queries`. It rejects zero-length, overflowing,
synthetic, short, and reused-fixture offset requests. Because only the selected
rows enter the query table, recall, latency, predictions, and future trace
artifacts remain aligned without task-local query copies.

Every physical benchmark row retains the parent query-file SHA and now also
records `query_offset` plus the SHA-256 of the exact selected line bytes. The
landmark/provenance row names the dynamic 1-based row range and carries both
parent and slice hashes. This makes Task 227's rows 201--400 calibration slice
and rows 1--200 blind evaluation slice suite-addressable and independently
attestable.

Validation: `artifacts/query-slice-tests.log` records five focused CLI tests,
all passing. They cover exact byte hashing, short-slice rejection, SQL
OFFSET/LIMIT expansion, suite command expansion, and rejection without a real
corpus. No PostgreSQL callback or benchmark behavior changed in this slice;
the trace and graph surfaces follow in separate checkpoints.

Please review offset/count validation, exact-byte digest semantics, SQL slice
loading, suite expansion, and normalized provenance fields.
