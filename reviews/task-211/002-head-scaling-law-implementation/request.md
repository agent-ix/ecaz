# Review request — Task 211 P1/P2: head scaling law implementation

- Task: `plan/tasks/211-ec-distann-head-scaling-law.md`
- Packet: `reviews/task-211/002-head-scaling-law-implementation/`
- Code commit: `4fe5d5c53` (`feat(distann): implement head sizing crown cache and fused hops`)
- Follow-up commit: `9c8f2aafb` (counter capture and seed-set provenance)
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

The PG18 library and benchmark-feature compiles pass. The focused deterministic
attestation/digest-binding test passes. The required real-corpus A/B sweep at
10k/50k/100k (recall, latency, storage) is not yet executed: the checkout and
host do not contain the staged corpus/query/manifest inputs, and suite audit
fails on those missing files. No synthetic or historical numbers are being
used as substitute evidence.

See `artifacts/manifest.md` and `artifacts/validation.log`.

## Status

Open — awaiting reviewer feedback and the required benchmark evidence.
