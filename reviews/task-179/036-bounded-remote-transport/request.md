---
task: 179
packet: 036-bounded-remote-transport
role: coder
status: review-requested
head: ceb15f73ac69fcd98896457c9578fadae2ff0c09
date: 2026-07-12
---

# Review request: bounded, interrupt-aware remote transport

Please review commit `ceb15f73a`, the exact-SHA validation evidence under
`artifacts/`, and the scoped decision in `verdict.md`.

This checkpoint responds to packet 025 reviewer finding P2. The requested
decisions are:

1. Does the common await wrapper bound every foreground remote RPC while
   checking PostgreSQL interrupts before and after each await?
2. Are connection establishment and server-side execution independently
   bounded by Userset connect and statement timeout GUCs?
3. Does a pooled session safely refresh `statement_timeout` when a backend
   changes the Userset GUC, while preserving the remote error as the primary
   diagnostic?
4. Do lifecycle, physical handoff, scan, and identity-setup call sites retain
   their existing error classification and conninfo redaction?

Key validation results:

- PG18 clippy with warnings denied passes;
- all seven focused transport unit tests pass, including client deadline,
  remote-error preservation, configured connect timeout, and redaction;
- the live PG18 pooled-session test passes after tightening the same session
  from 10 seconds to 10 milliseconds and asserting cancellation within two
  seconds; and
- the existing PG18 three-owner physical-handoff test passes under the new
  wrappers.

This request is scoped to the packet 025 transport P2. Task 179 remains open
for real three-instance partial-window fault injection, head-cap sensitivity,
the epoch-mismatch retry question, and outstanding outside review/Task 172
evidence.
