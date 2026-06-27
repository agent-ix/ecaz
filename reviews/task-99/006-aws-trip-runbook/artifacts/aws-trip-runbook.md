# Task 99 AWS trip runbook — Graviton 4 + Intel lanes (single trip)

Preconditions (all must hold before `cloud up`):

- [ ] Task 99 branch merged to main (incl. packet 004 `ecaz.isa_cap` —
      a dispatcher change; single-trip economics forbid landing it
      after the trip).
- [ ] Local validation (packet 003) green: suite audit clean, recall
      byte-equal at every on/off pair, counter attribution sane.
- [ ] Release backend discipline understood: `cloud install` builds
      from the git ref; never run cargo test between install and bench
      without re-install (pg_test debug-install trap).
- [ ] Operator go on spend (~$5–8 total, est. 6–9 instance-hours/lane).

Corpus base: `snap-0e9c7743263e61d70` (real DBpedia 10k/50k/100k/1M +
GT; the only surviving snapshot — never destroy without a fresh
snapshot; `cloud down` enforces this).

## Lane A — Graviton 4 (`10k-medium`, m8g.2xlarge, Neoverse V2/SVE2)

1. **Provision + install**

       ecaz cloud up --profile 10k-medium \
         --from-snapshot snap-0e9c7743263e61d70 --git-ref <main-sha>
       ecaz cloud install --profile 10k-medium --git-ref <main-sha>

   Record in manifest: instance type, region, on-demand $/hr (both
   lanes — the AC4 price/performance comparison requires it), main SHA,
   installed backend SHA + `ecaz_build_profile()` probe.

2. **Day-one smoke set** (on-host, before any bench; logs packeted).
   Focused `cargo test --lib` filters (each verified pg_test-safe):
   `lut32` (incl. the NEON transpose unit test), `qjl32_` (SVE2 test
   must assert `Isa::Sve2` and report measured vector lanes — expected
   `sve2-128` / 4 lanes; **stop condition** if `Isa::Sve` or no lane
   count), `rabitq32`, `grouped_pq`, `hamming32`, `int8_approx32`,
   `candidate_batch`, `quant::isa` (cap table tests).
   Then **re-run `ecaz cloud install`** (or verify backend SHA
   unchanged) before benching — cargo test may have touched artifacts.

3. **Lane sources**: `ecaz corpus list` to discover the snapshot's
   corpus tables; write `t99-fixture-sources-aws.sql` (two CREATE TABLE
   AS statements mirroring the local sources file — embeddings are raw
   f32, any profile's corpus table is a valid source); commit it to the
   trip packet. Then run shared
   `reviews/task-99/002-profile-suiteconfig/artifacts/t99-fixtures.sql`
   (~11 × 100k index builds; est. 2–4 h; `pause`/`resume` if split
   across sessions).

4. **Main profile run** (FR-038):

       ecaz bench suite run \
         --config reviews/task-99/002-profile-suiteconfig/artifacts/task99-profile-suite.json \
         --artifact-dir reviews/task-99/<g4-packet>/artifacts \
         --manifest-output .../suite-manifest.json \
         --results-output .../results.jsonl

   QJL generate/load steps re-create the 1024-dim fixture on-host.
   Suite preflight records the backend; refuse-on-debug guard active.

5. **NEON-capped pass** (the SVE2-vs-NEON measurement):

       ecaz bench suite run \
         --config reviews/task-99/004-isa-cap-dispatch/artifacts/t99-g4-neon-cap-suite.json \
         --artifact-dir reviews/task-99/<g4-packet>/artifacts ...

   Expected: counter rows flip to `isa=neon`; recall byte-equal vs the
   uncapped cells for bit-exact families. Gather per-family
   SVE2-vs-NEON ns/candidate — feeds ADR-077 §6 at ACCEPT time.

6. **Task 97 runbook cells** (closes Task 97): per
   `reviews/task-97/022-graviton4-closeout-runbook/`, run its suite
   config kernel_on/kernel_off passes; counters must show
   `quant=turboquant_qjl isa=sve2` + measured vector length.

7. **Task 94 G4 evidence** = the profile's grouped-PQ cells (IVF
   pq_fastscan on/off, DiskANN prefilter=grouped_pq on/off) — annotate
   as gather-shape SVE2 kernel (repack deferred). Optional: 10k/25k
   IVF pq_fastscan replicas if Task 94's reviewer asks.

8. **Teardown**:

       ecaz cloud snapshot --profile 10k-medium \
         --description 'post-task99-g4-profile: t99 fixtures + corpus base, main=<sha>'
       ecaz cloud down --profile 10k-medium --yes

## Lane B — AWS Intel (`10k-intel`, m7i.2xlarge, AVX2)

Same sequence minus the SVE-specific items:

- Day-one smoke expects `Isa::Avx2`; no vector-lane capture.
- No NEON-capped pass (capping to NEON on x86 lands on scalar by
  design — no kernel to measure).
- No Task 97/94 runbook extras (their Intel evidence is closed:
  Tasks 97 packet 026 / 103 packets).
- Same sources discovery → same `t99-fixtures.sql` → same main profile
  config → snapshot → down.

## Operational notes (from project memory)

- Don't rely solely on session notifications for long cloud steps —
  check AWS-side state on a cadence (SSM ConnectionLost may not fire
  local notifications).
- Everything suite-driven; no ad hoc SSM command soup; the manifest's
  re-run section must be reproducible.
- One index per replicated table (planner picks by cost, not GUC).
- Never leave the DB running after the lane completes.

## Outputs

Two run packets (G4, Intel) with suite manifests, results.jsonl,
counter rows, instance/pricing records, day-one smoke logs — feeding:
the AC4 per-ISA comparison + price/performance table, the AC3
decoupling map, the ADR-025 TQ-mode reevaluation dataset, the ADR-077
§6 SVE2-vs-NEON record + §4 IVF default decision, and the Task 94/97
closeouts.
