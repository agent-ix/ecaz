---
task: 231
packet: 003-format-and-reader
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 01
---

# Task 231 fixed-stride byte format checkpoint

Review code checkpoint `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`.
This is the first narrow Packet 003 slice; the persisted descriptor, raw
relation I/O, and production read-path switch remain open and will be requested
as later sequences in this packet.

The new `fixed_stride` module implements the Packet 001 arithmetic and byte
contract without PostgreSQL backend dependencies:

- checked dense-ordinal addressing for packed, exactly-one-page, and aligned
  multi-block records;
- re-derivation validation for every persisted layout value;
- SHA-256-bound 80-byte page envelopes and 80-byte node headers;
- exact vector, search code, adjacency ids/codes, row locator, tombstone, and
  canonical padding in one fixed-stride node;
- version-first admission, generation/ordinal/vec-id binding, per-page and
  per-node corruption rejection, and pooled decode buffer reuse.

Focused validation is 5/5 green. It exercises all three page-fit classes,
address overflow, persisted-arithmetic drift, byte-exact node round-trip,
wrong version/identity, digest and padding corruption, every multi-block
segment, and build-identity binding. See
`artifacts/fixed-stride-format-tests.log` and its packet manifest.

Please review the checked arithmetic, version-first decode ordering, digest
coverage, padding rules, and whether the pure format API is suitable for the
next relation/WAL slice.
