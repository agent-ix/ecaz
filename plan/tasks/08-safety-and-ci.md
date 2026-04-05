# Task 08: Safety and CI

Status: partially started

Progress notes:
- Build/test/lint targets exist in the repo.
- CI wiring, fuzzing, and formal unsafe-audit enforcement remain pending.

## Scope

Enforce safety, stability, licensing, and CI verification across the repo.

## Owns

- `NFR-004`
- `NFR-005`

## Dependencies

- Can start immediately

## Unblocks

- safe parallel development
- repeatable merge gating

## Deliverables

- CI jobs for fmt, clippy, unit, pg tests, and license checks
- fuzz harnesses and panic-resistance checks
- unsafe comment audit enforcement

## Primary Tests

- `TC-035`
- `TC-036`
- `TC-118`
- `TC-119`
- CI gates for `NFR-005`

## Notes

- This task should run continuously in parallel with implementation, not as a final cleanup pass.
