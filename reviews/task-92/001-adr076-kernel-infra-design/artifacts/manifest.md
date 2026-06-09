# Task 92 Packet 001 Artifact Manifest

- head SHA: `5cdcf38a529ddec50665a4ea44b806f03383897f`
- task bucket: `reviews/task-92/`
- packet path: `reviews/task-92/001-adr076-kernel-infra-design/`
- timestamp: `2026-06-09T02:51:39Z`
- scope: design-only ADR-076 and block-kernel skeleton fit audit
- lane / fixture / storage format / rerank mode: not applicable; no benchmark or pg_test run
- isolated one-index-per-table vs shared-table surfaces: not applicable; docs-only packet

## Artifacts

### `spec/adr/ADR-076-universal-block-kernel-pattern.md`

- command context:
  - `sed -n '1,260p' plan/tasks/92-cross-quant-block-kernel-infrastructure.md`
  - `sed -n '1,260p' src/am/common/candidate_batch.rs`
  - `sed -n '1,220p' src/am/common/quant_codec.rs`
  - `sed -n '1,220p' src/quant/lut32.rs`
- result: proposed ADR drafted
- key cited decisions:
  - universal block width is 32 candidates;
  - dispatch routes through `QuantCodec::score_ip_batch`;
  - module layout is `src/quant/<kernel>/{mod.rs,scalar.rs,neon.rs,sve.rs,avx2.rs}`;
  - runtime ISA detection covers `Scalar`, `Neon`, `Sve`, `Sve2`, and `Avx2`;
  - SVE/SVE2 must be vector-length agnostic; Graviton 4 is the ARM target and
    uses the `Sve2` dispatch branch when available;
  - scalar is bit-exact, SIMD gets ADR-076 ULP/relative tolerance.

### `skeleton-fit-audit.md`

- command context:
  - `rg -n "QuantCodec|score_ip_batch|CandidateMeta|TurboQuantExactScoreMode|DiskannPreparedPrefilter|SpirePreparedAssignmentScorer" src/am src/quant`
- result: all seven Task 92 in-scope quant families fit the skeleton
- key cited result:
  - Task 91 Phase 2 grouped-PQ model binding is the only prerequisite before
    Task 92 implementation starts.

### `counter-and-suite-contract.md`

- command context:
  - `rg -n "candidate_batch_scoring|CandidateBatchScoringSnapshot|lut32_flushes" src crates`
  - `rg -n "bench suite|SuiteConfig|quant" crates/ecaz-cli/src/commands/bench crates/ecaz-cli/suites src`
- result: Phase 2 counter contract and Phase 5 suite-axis contract added
- key cited decisions:
  - introduce `ec_block_kernel_scoring_*` SQL functions while keeping Task 87
    compatibility wrappers;
  - emit `[block-kernel-counters]` CLI lines with `(surface, quant, isa)`;
  - require explicit suite cell statuses: `valid`, `missing_kernel`,
    `structurally_absent`, and `invalid_config`.

## Validation

No tests run. Task 92 Phase 1 is explicitly design-only and has no Rust code
changes other than the proposed ADR and review artifacts.
