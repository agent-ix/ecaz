# Review request — Task 213 P1/P2: fused head-hop implementation

- Task: `plan/tasks/213-ec-distann-fused-head-hop.md`
- Packet: `reviews/task-213/002-fused-head-hop-implementation/`
- Code head: `cc6a01c66` (`Expose crown width activation provenance`)
- Date: 2026-08-01. Coder: Codex

## Reviewer follow-up

The reviewer’s requested changes are implemented:

- the unfused crown path uses the full head fan-out, while the fused path
  returns crown seeds and records fused-hop activation;
- the seed digest probe sets the exact arm GUCs on the coordinator;
- existing scan tests cover positional restart, threshold, and first-round
  semantics;
- physical custom-scan search now retries typed epoch-mismatch failures by
  discarding stale generation/crown state and reopening the active epoch.

## Validation and evidence

PG18 checks and four focused crown-cache tests passed. The final
`ecaz bench suite` completed all six unfused/fused steps at 10k/50k/100k.
Fused provenance marks the seed-set change, crown fallbacks remain zero, and
the fused-hop counters are nonzero on every scale. Results and storage
provenance are summarized in `artifacts/manifest.md`; structured results are
in `artifacts/bench-run-final2/results.jsonl`.

Status: open — reviewer follow-up is complete; awaiting outside reviewer
acknowledgement.
