---
task: 234
packet: 005-50k-reuse-drift-bound
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 234 reviewer-requested 50k reuse drift bound

This packet responds only to the remaining item in packet 004 feedback. No
fault matrix or 10k/100k fixture was rerun, and the accepted TLS, correctness,
recall, and storage findings are unchanged.

The committed suite now has one 50k reuse repeat for each arm. Because the
original stopped fixtures had already been removed, each arm first created a
fresh seed fixture and then immediately ran the requested
`reuse_fixture: true` step against it. Both repeats passed strict provenance at
50,000 rows: control `ed5ac814c05350ca695533fcd54d0df11faa876b` and candidate
`7c42dc818e80fe68246dc5c45255640b81c551b1`, release PG18, verify-full TLS,
the registered query slice, and `persisted_head`.

The control warm mean moved 8.83 to 8.47 ms on same-fixture reuse (-4.08%);
the candidate moved 9.43 to 8.74 ms (-7.32%). The candidate/control reuse-only
delta is +3.19% (p50 +2.03%, p95 +0.96%, p99 +1.89%). Today's two control
observations are +8.21% and +3.80% relative to the old 8.16 ms control. This
directly bounds fixture/run drift across the historical +7.1% screened signal.
It also retires packet 004's -6.37% "faster" headline: on the matched fixture
the candidate is slower at both the fresh-seed position (+6.80% mean) and the
immediate-reuse position (+3.19% mean), and all four reuse statistics have the
same positive sign. The fixture cannot resolve the exact magnitude of this
small cost, but the measured direction is consistently slower and matches the
added deadline/cancellation mechanism.

The benchmark runner needed two narrow corrections to execute the registered
repeat honestly: secure reuse now reloads the existing fixture TLS artifacts,
and `skip_fault_drills` now skips the mutating routed DELETE/VACUUM drill it
already documented as excluded. The strict row-count check was not weakened.
`cargo build -p ecaz-cli` passed (one inherited dead-code warning), and the
successful suite runs exercise both corrections end to end.

Durable evidence is in
`benchmarks/task234-current-tls-read-rpc-cancellation-ab/`, especially its
updated `manifest.md`, the two `suite-manifest-*-drift-v2.json` files, the two
`suite-results-*-drift-v2.jsonl` files, and the four seed/repeat summaries and
latency logs under `artifacts/run/`.

Please record an explicit product ACCEPT/REJECT ruling on the measured cost and
then record the final Task 234 disposition. The coder recommendation is ACCEPT:
the P0 deadline/cancellation property is worth the small cost, whose exact
magnitude is not separable from fixture drift and is well below the historical
+7.1% screened result. This is a cost acceptance, not a parity or speedup claim.
