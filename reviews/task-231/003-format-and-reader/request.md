---
task: 231
packet: 003-format-and-reader
agent: Codex
role: coder
model: gpt-5
date: 2026-08-29
seq: 02
---

# Task 231 fixed-stride format and persisted selector checkpoint

Review code checkpoints `1e0d5906abfa5a586091ca51b4ccf1a48690f37f`
and `c644b3fb0cc7bad7027bd51d277a6578e69b81c1`. This is the second narrow
Packet 003 slice; raw relation I/O and the production read-path switch remain
open and will be requested as later sequences in this packet.

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

## Sequence 02: persisted admission selector

The second checkpoint adds:

- generation descriptor V5 with a digested
  `DistannFixedStrideLayoutDescriptorV1` and graph-record discriminator V3;
- decode admission that reads the descriptor version before its version-sized
  tail, re-derives every arithmetic field from the admitted codec artifact,
  dimensions, degree, and code stride, and rejects drift;
- opt-in `node_storage_layout='fixed_stride'`, defaulting to `graph_heap`, legal
  only for distributed control indexes and mutually exclusive with Tasks
  229/230; and
- build registration resolution that freezes the descriptor from the actual
  workspace codec stride rather than trusting user-supplied byte counts.

The combined format/descriptor gate is 6/6 green with no compiler warnings;
see `artifacts/fixed-stride-descriptor-tests.log`. Durable crash/replay binding
of the selector belongs to Packet 004 and is not claimed here.
