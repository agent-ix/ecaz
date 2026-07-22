---
task: 196
packet: 002-identity-reuse
role: coder
status: review_requested
date: 2026-07-22
seq: 1
---

# Task 196 identity-keyed reuse fix and real-100k semantics

This checkpoint fixes the stable-prefix duplicate request reproduced in packet
001. Previously materialized remote payloads are now reused by immutable vec_id
within the proven prefix instead of by raw rank. Deeper search still defines
the current order; the patch does not preserve or impose an earlier ordering.

The fixed release binary passed the full real-100k semantic drill. All nine
cases report `pass=true` and `duplicate_requested=0`: fewer than, exactly one,
and more than one lazy window; first- and multiple-window rejection; NULL;
external TOAST projection/qual; mixed local/remote ownership; and owner failure
after the first batch.

The formerly failing multiple-rejection case now returns the same 10-row digest
as eager materialization after 48 payload reads (31 remote, 17 local), with no
remote vec_id requested twice. The mixed-owner case returns 7 remote plus 3
local rows. The later owner-outage drill preserves its expected error digest
and also reports zero duplicates.

Focused unit coverage simulates equal-distance rank swapping and proves both
payloads remain associated with their own immutable IDs without consuming an
unmatched pending payload. The focused PG18 test passed; `cargo check -p
ecaz-cli` passed with one pre-existing dead-code warning.

Please review the identity boundary, the fact that new search order remains
authoritative, and the nine scenario results. Packet 003 will carry the
required isolated 10k/50k/100k production recall/latency/storage A/B before
Task 196 closeout.
