# Review request — Task 213 P0: fused head hop spec

- Task: `plan/tasks/213-ec-distann-fused-head-hop.md`, phase P0 (spec-first)
- Packet: `reviews/task-213/001-fused-head-hop-spec/`
- Spec artifact: `spec/functional/distann/read/FR-090-distann-fused-head-hop.md`
  (commit `fbf07d7ea`, branch `task-203-ec-distann-conformance`)
- Date: 2026-08-01. Coder: fable (Claude Fable 5)

## What to review

FR-090 specs removing the dedicated head fan-out RTT by fusing seed
selection into the first traversal expansion: crown codes (FR-089) answer
the candidate half at the coordinator; exact seed distances return with the
first owner expansion — TRAV-30's candidate/result split one layer up. The
head hop is removed by fusing it with the next hop, never by answering from
resident state (the conformance distinction keeping this outside FR-084
territory).

Invariants bound simultaneously, per the task file:

- FR-079-AC-1 positional reassembly on the fused first request;
- Algorithm-1 candidate/result split + Task 205 threshold semantics on the
  fused expansion;
- NFR-021: nothing new resident beyond the FR-089 crown;
- unfused two-phase fallback with identical results (accelerator with a
  correct slow path, never the only path);
- seed-digest honesty: exact seed policy holds, or the arm is labeled a
  seed-set change and measured as one — never silently both;
- `fused_head_hops` activation counter asserted non-zero; hop/RTT counters
  reported alongside latency (mechanism visible in evidence).

Entry gate preserved: Task 212 P1 (crown exists, counters prove activation)
gates FR-090 implementation; the A/B design (fused vs unfused, both arms
crown-on) attributes the delta to fusion alone; predicted win ~one RTT
(~2–3 ms at 10k).

## Validation

`quire validate` clean (advisory EARS warnings only). Implementation NOT
started.

## Status

Open — awaiting reviewer feedback.
