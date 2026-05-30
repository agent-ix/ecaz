# Task 67 Review Request: measured closeout amendment

## Summary

This packet amends `plan/tasks/67-rabitq-intel-avx-optimization.md` to align closeout criteria with the packet-local Intel measurements already accepted or recorded:

- kernel-layer targets pass in packets 020/023
- bits=1 SQL reaches 3x at the recall-preserving `nprobe=64` operating point, but not at `nprobe=16/32`
- bits=8 SQL headline evidence exists for all four `rabitq8*` variants in packet 027 and shows the strict 4x SQL threshold fails because scoring is about 1% of total wall time
- bf16 decision evidence in packet 029 shows `rabitq-bf16` should stay disabled by default
- AVX2 fallback code remains required and differential-tested, but no AVX2-only host benchmark is required for closeout unless such a host becomes available before closure

## Why

Packet 025 feedback correctly required either more measurement or a task amendment for the strict SQL and AVX2-only-host gates. Subsequent packets supplied the missing measurements and showed the SQL thresholds are bottlenecked outside the scorer. This amendment makes that measured outcome explicit in the task definition instead of closing against failed original thresholds.

## Code Under Review

- `plan/tasks/67-rabitq-intel-avx-optimization.md`
- code/doc commit: `673f14eac6346fee531c364fe1181392b28bed1c`

## Validation

This is documentation only. Packet-local artifacts are under `artifacts/local/`; see `artifacts/manifest.md`.

- `git show --stat --oneline 673f14eac`: confirms one-file task amendment
- `git show -- plan/tasks/67-rabitq-intel-avx-optimization.md`: captures the exact amendment diff

## Notes

No benchmark or test pass is claimed in this packet. The closeout audit packet will cite the amended task criteria and the measurement/test packets that prove them.
