# Excluded remediation-after attempt

On 2026-07-14 PDT, the exact remediation-after suite was interrupted during its 50k step at the operator's request because concurrent database maintenance would taint the measurements.

- Installed extension: `45491d1052ef0369a9f418b055b462663cf5612c`, release profile.
- The 10k step had completed and the 50k step had reached physical serving, but the maintenance overlap makes the attempt unsuitable as a decision-grade arm.
- The suite process and all three fixture PostgreSQL instances on ports 40720-40722 were stopped.
- All generated `remediation-after/` artifacts, the partial suite log, and `target/task179-final-signoff-exact-ab` run state were removed.
- No number from the interrupted attempt is cited by this packet.

A fresh, non-resumed 10k/50k/100k candidate arm subsequently completed after maintenance. The retained evidence lives under `remediation-after/` and passes post-prune status/report/audit. This file remains only as the exclusion record for the discarded attempt.
