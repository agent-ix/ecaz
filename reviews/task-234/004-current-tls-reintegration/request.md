---
task: 234
packet: 004-current-tls-reintegration
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 234 current-TLS reintegration and closeout gates

This packet requests outside review of Task 234 after reintegrating the read
RPC deadline/cancellation contract onto the accepted Task 236 secure transport
base. The implementation preserves `RemoteTlsConfig`, security-fingerprinted
pool keys, verify-full mutual TLS, typed transport failures, and the existing
secret-redaction boundary. It does not reintroduce pre-Task-236 `NoTls` paths.

All five read/control RPC families use the common bounded await contract.
Client deadline and PostgreSQL interrupts trigger bounded cancel delivery and
eviction of ambiguous sessions; completed remote statement timeout/query
cancel outcomes retain only safe state. Multi-owner failures remain fail-closed
and request-order deterministic. The successful-batch path skips unnecessary
error normalization, which is the screened fast path carried into this
candidate.

The current-TLS PG18 release+`pg_test` matrix passed 25 of 25 secure cells:
physical head search, crown export, gateway export, head-shard export, and
head-shard import crossed with remote statement timeout, local backend cancel,
local statement timeout, remote backend termination, and connection reset.
Every cell returned an error instead of partial rows, drained remote work, and
completed a positive clean retry. The slowest reset was 1.223 seconds under a
5-second bound; every other cell completed in at most 509 ms under a 2-second
bound.

The first secure run found a CLI fixture bug: Task 234's connection-reset arm
restarted the owner without its TLS server configuration. The harness now
restarts physical nodes through the selected transport fixture. The retained
failed-run diagnostic proves the failure mode; the final passing matrix is the
acceptance source.

Focused validation passed:

- `cargo check --no-default-features --features pg18`;
- `cargo check --no-default-features --features pg18,pg_test`;
- `cargo check -p ecaz-cli` (only a pre-existing dead-code warning);
- `cargo test --lib --no-default-features --features pg18
  remote_transport::tests` (15/15); and
- repository formatting is clean.

The required fresh 10k/50k/100k `ecaz bench suite` A/B is in
`benchmarks/task234-current-tls-read-rpc-cancellation-ab/`. Recall is identical
at 10k/50k. At 100k one of 2,000 expected hits moves at the top-10 boundary
(0.9295 to 0.9290), while head membership remains byte-identical. Storage is
identical or within one 8 KiB page. Candidate warm mean latency is 3.84%,
6.37%, and 4.52% lower at 10k, 50k, and 100k respectively; every recorded
p50/p95/p99 is also lower. Because the arms ran sequentially, this supports a
neutral/no-regression disposition rather than a broad performance claim.

The old pre-TLS candidate's negative latency result remains historical evidence
in the Task 234 ledger. It is superseded for production disposition by this
exact-current-base secure run, not erased.

Please review the TLS preservation and pool-key boundary, interrupt/cancel race
handling, typed error and eviction decisions, secure restart harness, 25-cell
fault evidence, and the one-result 100k recall boundary delta. The coder
recommendation is ACCEPT, but Task 234 remains review-open until an outside
reviewer records a verdict. Tasks 237 and 228 remain blocked on that verdict.
