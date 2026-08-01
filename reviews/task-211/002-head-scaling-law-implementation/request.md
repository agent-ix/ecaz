# Review request — Task 211 P1/P2: head scaling law implementation

- Task: `plan/tasks/211-ec-distann-head-scaling-law.md`
- Packet: `reviews/task-211/002-head-scaling-law-implementation/`
- Code commit: `4fe5d5c53` (`feat(distann): implement head sizing crown cache and fused hops`)
- Follow-up commit: `9c8f2aafb` (counter capture and seed-set provenance)
- Final code fix: `0a526ac1e` (exercise crown seeds in plain arms)
- Date: 2026-08-01. Coder: Codex

## What to review

This checkpoint implements the P1 build-side law and the CLI plumbing needed
for the P2 matrix:

- `head_sampling_rate`, `head_cap_floor`, and `head_cap_ceiling` reloptions;
- deterministic `ceil(rate * captured_record_count)` resolution with frozen
  domain validation and trained-policy rejection unless the resolved cap is
  4096;
- v3 generation-descriptor attestation containing the resolved capacity,
  captured count, law inputs, and sample count, all covered by the generation
  digest;
- `ecaz bench suite` forwarding of the build-time law controls and provenance
  fields in build/recall/latency lines.

## Validation

PG18 library and benchmark-feature compiles pass, and the focused deterministic
attestation/digest-binding test passes. The required `ecaz bench suite` A/B
matrix completed at 10k/50k/100k with recall, latency, and storage evidence.

| scale | control recall / ms | law recall / ms | storage ratio |
| --- | --- | --- | --- |
| 10k | 0.9940 / 39.00 | 0.9940 / 37.90 | 1.235867 |
| 50k | 0.9595 / 51.80 | 0.9595 / 51.00 | 1.332667 |
| 100k | 0.9145 / 53.00 | 0.9145 / 52.00 | 1.351147 |

The 10k/50k arms used extension SHA `d4c39c8218055195eed559249116251bf0315f73`;
the 100k arms used the final installed SHA `0a526ac1eb840a975ac00130201058b187f4057d`.
The packet-local `results.jsonl` is the structured source of truth.

See `artifacts/manifest.md` and `artifacts/validation.log`.

## Status

Open — implementation and benchmark evidence complete; awaiting outside reviewer feedback.
