# Checkpoint-1 reviewer carry-in disposition

Source commit `255081d74aa6ce430a2a21ee5555e9569c0a0fa7` processes the
five carry-ins from `feedback/2026-08-26-01-reviewer.md`.

| Carry-in | Disposition |
| --- | --- |
| 1 — registration identity | Closed. No-cover registrations retain exact V1 bytes and the existing golden digest. Covered registrations use conditional V2 bytes that add the domain-separated cover-descriptor digest. T1 create/replay and T2 replay recompute the resolved cover digest, so reloption drift changes the expected registration digest and errors before source capture. |
| 2 — bound test name | Closed. Renamed to say the 258-byte bound is *attained*, not that the defense-in-depth overflow arm is reachable. |
| 3 — unconditional indexed attnum | Deliberately retained. Seq-01 proved it cannot reject a previously usable no-cover generation, and early validation gives one T1/T2 shape for covered and uncovered builds. |
| 4 — clippy | Closed. Strict all-target clippy reports exactly the four inherited main lint failures; the four files are unchanged. Allowing only those exact lint names makes all targets pass with `-D warnings`, proving no Task 229 lint addition. |
| 5 — identity INCLUDE attnum | Not excluded. The accepted contract permits any supported non-vector scalar, and source identity can be a legitimate projected result. Redundant coverage is bounded and explicit; silently rejecting it would narrow the declared contract without a correctness reason. |
