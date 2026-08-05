# Review request — Task 213 P1/P2: fused head-hop implementation

- Task: `plan/tasks/213-ec-distann-fused-head-hop.md`
- Packet: `reviews/task-213/002-fused-head-hop-implementation/`
- Code head: `a8b1699528e593b45f55fc25329199714d4627ff` (`test(distann): verify crown fallback and lifecycle`)
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

Status: complete pending outside reviewer acknowledgement. The fused consumer
is active and measured; the shared capacity matrix selects 2048 entries for
the opt-in configuration. Production defaults remain `crown_capacity=0` and
`fused_head_hop=off` because the measured fused arms are explicitly labeled
`seed_set_change=true`.

The shared capacity evidence and exact 512/2048/4096 × 10k/50k/100k table are
in `reviews/task-212/002-crown-cache-implementation/artifacts/capacity-matrix-summary.md`.
