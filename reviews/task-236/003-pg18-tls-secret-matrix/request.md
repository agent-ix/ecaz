---
task: 236
packet: 003-pg18-tls-secret-matrix
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 236 PG18 TLS and secret-rotation matrix

This packet requests outside security/code review of the live PG18 matrix at
`45f895d68fb208f6b3cfe4d8b0fdd17d04c6b3db`. It does not request Task 236
closeout: the required 10k/50k/100k performance evidence and packet 004 outside
security closeout remain open.

## Implementation checkpoints since packet 002

- `eb09167bf` adds a private-CA, verify-full mutual-TLS PG18 fixture while
  preserving explicit loopback plaintext only for operator/setup traffic.
- `025ceb528` adds a sanitized pg_test TLS probe that returns stable categories
  rather than driver/server text.
- `5e77e51d2` makes live secret rotation backend-local instead of mutating the
  process environment after transport workers exist.
- `2688e56e9` adds the suite-driven certificate, handshake-fault, rotation, and
  pool-reuse matrix.
- `78769fdde`, `4b7953ac9`, and `253168e98` encode live-discovered fixture
  preconditions: three owners, isolation from unrelated publish-fault drills,
  and the generated schema of the test secret hook.
- `45f895d68` adds a live zero-leak assertion over catalogs, serialized
  generation descriptors, EXPLAIN, and topology status.

## Result

The three-owner physical fixture built and published under verify-full mutual
TLS, served 10/10 rows, and committed a routed insert on a remote owner. All 13
matrix cells passed:

- valid TLS 1.3 mutual authentication;
- wrong CA and wrong hostname;
- missing, incorrect, expired, and not-yet-valid client certificates;
- plaintext rejected by the TLS-only remote role;
- unsupported downgrade-capable sslmode rejected before connection;
- exact-peer connection reset, followed by clean TLS recovery;
- secret rotation with pool replacement, stable pool cardinality, and pooled
  reuse;
- live and artifact-level secret-exposure checks with zero matches.

The observed rotated handshake took 466 ms and the immediate pooled reuse took
3 ms. These are diagnostic point measurements, not the still-pending scaled
10k/50k/100k performance matrix.

See `artifacts/manifest.md` for exact commands, provenance, and artifact paths.
Please focus review on certificate generation/validity cells, the second
loopback hostname-verification proof, HBA plaintext rejection, exact-peer reset
at connection establishment, rotation eviction semantics, and whether the live
and packet-level leak surfaces are sufficient for packet 004.
