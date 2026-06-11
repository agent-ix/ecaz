# Review request — Task 99 packet 002: index × quant × mode profile SuiteConfig

- Task: 99, item 9 (the pinned G4 + AWS-Intel profile)
- Coder: Task 102/103 author lane
- Date: 2026-06-11
- Head: post-Task-104 main (uses the runnable `retired` kernel_status)

## What this packet contains

The profile design (`artifacts/t99-profile-design.md`), the generator
(`gen_t99_profile.py`) and generated SuiteConfig
(`task99-profile-suite.json`, 91 steps / 45 bench cells), the per-lane
fixture bootstrap SQL (`t99-fixture-sources-local.sql` +
shared `t99-fixtures.sql`), and the clean dry-run
(`suite-dry-run.log`, all 91 steps resolve, no errors).

Local validation (full execution on the Intel desktop) runs next as
packet 003; the AWS trip waits for that plus review of this packet.

## Specific review asks

1. **Cell matrix completeness/correctness** vs the design requirements
   pinned in the task file (dimension coverage, markers, batch axis):
   anything missing or mis-marked? In particular:
   - IVF rabitq bits=4 as `missing_kernel` (real storage lane, no
     kernel by Task 93 scope) vs `structurally_absent`;
   - HNSW exact mode as `structurally_absent` (no-kernel f32 baseline).
2. **Fixture shapes**: index reloptions mirror task87/94/102/103
   conventions (see SQL headers) — flag any reloption that would make a
   cell non-comparable to its per-family closeout packet.
3. **Table-replication bootstrap** (raw-f32 portability argument in the
   design doc §3) — sanity-check the claim and the AWS implication
   (snapshot's IVF-profile tables can seed all AMs).
4. **Trip plan** (§AWS): instance pairing m8g.2xlarge vs m7i.2xlarge,
   single-trip sequencing incl. Task 97 runbook cells riding the G4
   instance.
