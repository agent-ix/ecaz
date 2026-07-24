---
task: 198
packet: 003-lifecycle-and-faults
head_sha: 4c09b9a99
extension_sha: 2ff72b3e49609c44cec881f72edf183a83554412
role: coder
date: 2026-07-23
status: open
---

# Review request: traversal-replica lifecycle and fault safety

Please review the lifecycle implementation through `4c09b9a99` and the
three-process PG18 evidence summarized in
[`artifacts/manifest.md`](artifacts/manifest.md).

The gate demonstrates:

- atomic build rollback after a real owner outage and partial stream;
- identity-bound idempotent activation;
- identical ordered results on normal and injected mid-replica failure paths;
- fallback from a deliberately truncated/corrupt Ready image;
- a dedicated coordinator connection committing `Ready -> Stale` before the
  first mutation attempt raises stable retryable SQLSTATE `40001`;
- exactly one failing guard attempt, followed by a normal Stale owner-path
  retry;
- fenced `Stale -> Retiring` and idempotent relation/catalog reclaim; and
- unchanged owner payload semantics for nulls, external TOAST, quals,
  projection, mixed ownership, deepening, and remote failure.

The 10k timing is only a smoke signal (21.80 -> 17.90 ms over two samples).
Packet 004 remains the causal performance gate.
