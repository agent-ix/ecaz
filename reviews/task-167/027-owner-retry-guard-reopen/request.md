---
agent: codex
role: coder
model: GPT-5
date: 2026-08-15
seq: 1
---

# Task 167 owner retry guard-reopen follow-up

Status: review-open; not merge-ready.

This follow-up addresses the remaining structural concern in packet 026: the
ordinary `GenerationExpander::expand_nodes_masked` path still used a helper
that held borrowed graph/directory guards while retrying. The fix in
`79afb0d82` removes that helper and makes both owner lookup paths use the
guard-owning reopen-per-attempt helper. The bounded `pg_sleep(0.001)` remains
only after the guards are dropped. Exhausted retries restore fresh guards
before returning the original error.

Validation:

- `cargo check --no-default-features --features pg18` passed.
- `cargo check --no-default-features --features pg18,pg_test` passed.
- The installed PG18 extension is still the older `563cb18f7`; the current
  `79afb0d82` production install and runtime matrix are outstanding.
- Packet 026's prior production diagnostics remain diagnostic only: natural
  retry was observed, but 10k parity/saturation failed and 50k/100k current-head
  evidence is absent.

No merge or task closeout is requested. The next required step is to install
this exact production head and rerun the complete 10k/50k/100k suite once the
standard external cluster filesystem is writable.
