# Review request — Task 212 P1/P2/P3: crown cache implementation

- Task: `plan/tasks/212-ec-distann-crown-cache.md`
- Packet: `reviews/task-212/002-crown-cache-implementation/`
- Code head: `a8b1699528e593b45f55fc25329199714d4627ff` (`test(distann): verify crown fallback and lifecycle`)
- Date: 2026-08-01. Coder: Codex

## Reviewer follow-up

The reviewer’s requested changes are implemented:

- plain crown-on now falls through to the full head fan-out, preserving the
  control seed set and result identity;
- the digest probe sets the exact arm GUCs on the coordinator;
- width activation is counted independently from actual shard removal, with a
  geometry that reaches nonzero pruning at 50k/100k;
- crown resident bytes are reported as a bounded, codes-only relation;
- crown selection uses a `HashSet` for membership;
- capacity-zero configuration clears the gauge and does not report fallback;
- focused tests cover deterministic selection, complete-set population,
  complete-shard attestation, epoch/capacity digest binding, and codes-only
  entries.

## Validation and evidence

PG18 checks and four focused crown-cache tests passed. The final
`ecaz bench suite` completed all nine control/crown/crown-width steps at
10k/50k/100k. Plain crown digests match control at every scale; width is
explicitly labeled as a seed-set-changing approximate arm. Recall and
storage evidence are summarized in `artifacts/manifest.md`; structured
results are in `artifacts/bench-run-final2/results.jsonl`.

Status: complete pending outside reviewer acknowledgement. The full fused
capacity matrix is complete at 512/2048/4096 × 10k/50k/100k; capacity 2048 is
selected for the opt-in fused configuration. The plain and pruning arms remain
result-neutral/no-effect findings as required by their attribution contracts.

The exact merged capacity table and provenance are in
`artifacts/capacity-matrix-summary.md`; the two final 100k suite records are in
`artifacts/bench-run-capacity-release-a8b169952/suite-manifest-r2.json` and
`results-r2.jsonl`.
