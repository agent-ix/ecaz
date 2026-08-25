---
task: 222
packet: 005-main-integration
agent: Codex
role: coder
model: gpt-5
date: 2026-08-24
seq: 01
---

# Task 222 clean-main integration and final review

This packet requests final outside review of Task 222 on branch
`integrate/task222-payload-projection`. The branch starts at exact current
`main` SHA `de28655a42d254c2ac7f181569f07b92de5f3fae` and contains only the Task
222 implementation, its task-scoped evidence, and three narrow tracking-ledger
updates. It excludes the old branch's stacked Tasks 223-238 planning history.

The reviewed functional sequence was replayed onto current main as:

- `c24e42fa0` — typed exact/all-column payload projection contract;
- `cf7d1ae95` — query-expression context correction;
- `034ddd064` — correctness and suite/fixture lane;
- `096be7716` — fixture-reuse attestation; and
- `bd5068101` — reviewer-requested `copyObject` executor-local plan copy.

The old `010a0accc` retry-snapshot dependency was deliberately not replayed:
current main already carries the stronger Task 167 lifetime implementation,
including optional refreshed snapshots and safe guard retention. During the
three textual conflicts in the suite, fixture, and options surfaces, current
main's Task 167 behavior was preserved and only Task 222's payload-projection
fields, pairing rules, GUC, and arguments were added.

Integration validation passed:

- `cargo check --lib --no-default-features --features pg18`;
- `cargo pgrx test pg18 test_distann_payload_projection_contract
  --no-default-features --features pg18`: 1 passed, 0 failed, 2,582 filtered.

Packets 002-004 remain the correctness and 10k/50k/100k decision evidence.
Packet 004 shows byte-identical ordered predictions, identical recall and
storage, and a 33.33%-40.41% warm-mean latency improvement. This packet asks
the reviewer to verify the current-main integration, the conflict resolutions,
and closure of seq-03's `copyObject` and production-default findings.

No formatter was run. The PR contains no formatting-only commit and no
repository-wide formatting delta.
