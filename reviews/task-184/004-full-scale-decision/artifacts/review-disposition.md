# Task 184 outside-review disposition

Feedback source:
`../feedback/2026-07-20-01-reviewer.md`.

Verdict: **ACCEPT** packets 001--004 and the PROMOTE-to-Task-191 decision.
No blocking finding requires a Task 184 code or benchmark rerun. Production
remains eager until Task 191 independently lands and validates ADR-085 D12.

| Review item | Disposition |
| --- | --- |
| P3: the repeated-MD5 wide value may remain compressed inline | Accepted. Task 191 now requires incompressible content and/or `STORAGE EXTERNAL` plus an assertion proving genuine out-of-line TOAST before semantic comparison. Packet 003 is described precisely as compressed-inline varlena coverage; it is not retroactively claimed as external-TOAST proof. |
| P3: deepening can re-fetch materialized-but-unconsumed slots | Accepted. Task 191 now requires scan-local carry-forward of stable-prefix payloads and a multiple-window assertion that a rebuild does not remotely request the same already-materialized `vec_id` again. This is bounded and correctness-neutral in Task 184, so it does not invalidate the measured decision. |
| Note: `output_merge` aliases `materialize_output_associate` | Accepted. Task 191 must remove the double-booking or emit machine-readable alias metadata; its final report may not add both as independent stages. Task 184's tables did not sum them. |
| Note: suite output creation self-dirties the runner descriptor | Accepted. Task 191 must capture runner provenance before writing tracked outputs and produce a clean final runner descriptor without a prose exception. Task 184's unanimous clean node-side extension attestation remains valid. |
| Process: Tasks 185--190 need separate review/stakeholder handling | No Task 184 change. Those tasks remain independent proposals; Task 184 advances only Task 191. Their review packets and stakeholder dispositions are outside this packet. |

Documentation checkpoint
`6d4e1da825c9e9d0bd4f84f0bbc66ec4685afcfe` updates Task 184's status,
Task 191's contract/implementation/evidence gates, the program roadmap, and the
task index. No tests or benchmarks were rerun because this checkpoint changes
only planning and review-disposition documentation; the accepted code and
measurement artifacts are unchanged.
