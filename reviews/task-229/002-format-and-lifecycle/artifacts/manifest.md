# Task 229 packet 002 artifact manifest

- Head SHA: `acc33c9f6203a20508005c830d0dc8a8d7b483b7`
- Base SHA: `3419c9c758bea7d9940b27d9afbcf9e627e84879`
- Task / packet: `reviews/task-229/002-format-and-lifecycle/`
- Checkpoint: 1 — reloption parse and fixed-width cover resolution
- Timestamp: 2026-08-26T05:27:47-07:00
- Lane / fixture / storage format / rerank mode: not applicable — static
  format preflight checkpoint; no index or fixture was built
- Isolated vs shared surfaces: not applicable — no table or index was created
- Source commit: `acc33c9f6203a20508005c830d0dc8a8d7b483b7`

## Artifacts

### `cargo-fmt-check.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0; formatting clean. Stable rustfmt reports only the repository's
  existing warnings that nightly-only import grouping options are unavailable.

### `cargo-check-pg18.log`

- Command: `cargo check --lib --no-default-features --features pg18`
- Result: exit 0; `ecaz` finished the dev profile successfully in 28.21s.
- Environment note: the sandbox's first attempt could not resolve crates.io;
  it produced no code result and was not retained. The recorded run used the
  ordinary approved network-enabled Cargo path.

## Test and benchmark scope

No test, PostgreSQL, pgrx, fixture, corpus, or benchmark command was run. Unit
coverage was added but not executed under the repository's no-tests-by-default
policy. This checkpoint makes no runtime or performance claim.

## Reviewer provenance — seq 01 (2026-08-26)

- Agent / role / model: Agent IX / reviewer / claude-opus-5
- Head SHA at review: `e417e3cfb0dd6cbc653869b5404da9ccf6db6958`
- Source commit reviewed: `acc33c9f6203a20508005c830d0dc8a8d7b483b7`
- Base SHA: `3419c9c758bea7d9940b27d9afbcf9e627e84879` (confirmed to be the
  real `git merge-base` of HEAD and main, not just a claimed base)
- Task bucket / packet: `reviews/task-229/002-format-and-lifecycle/`
- Review scope: checkpoint 1 only — reloption grammar, fixed PG18 scalar
  allowlist and schema resolution, 258-byte bound, no-cover behavior, T1/T2
  preflight placement. Packet-002 items 2-5 were not required to exist and were
  not reviewed.
- Lane / fixture / storage format / rerank mode: not applicable — static
  code review of a preflight-only checkpoint; no index, corpus, or fixture
  was built or read
- Isolated vs shared surfaces: not applicable — no table or index was created
- Verdict: **DONE**; checkpoint 2 authorized, with five carry-ins recorded in
  the feedback file (item 1, binding the resolved cover into the registration
  identity, must be closed in checkpoint 2)

### `reviewer-seq01-verification.log`

- Command: read-only static inspection via `git show`, `git diff`,
  `git merge-base`, `grep`, and file reads against the worktree at
  `/home/peter/dev/ecaz/.worktrees/task229-covering-payload-sidecar`
- Timestamp: 2026-08-26
- Result: nine checks X1-X9 — scope/base confirmation, reloption registration
  and struct-layout sizing, grammar arms vs the packet 001 contract, the
  eleven-type allowlist and its binary send widths, the thirteen fail-closed
  resolution gates, the 258-byte arithmetic, T1/T2 preflight ordering under
  the control and registry locks, no-cover reachability analysis of the new
  unconditional `indexed_ecvector_attnum` call, and a cross-check of the
  coder's `cargo fmt` / `cargo check` logs against `request.md`
- Key result lines cited by the feedback file: X1 (source commit touches only
  `src/am/ec_distann/**`; declared base is the real merge-base), X4 (allowlist
  set-identical to `request.md:36-38`, all eleven send widths correct), X5
  (all six accepted exclusions present, every arm returns
  `EC_SCHEMA_UNSUPPORTED`, no panic or silent skip), X6 (16 x 16 + 2 = 258
  attained exactly; the `> MAX_*` guards are unreachable), X7 (`control` is
  the index relation itself, so the T2 `relation_options` hoist is
  value-preserving), X8 (no index that previously produced a usable no-cover
  generation now fails)
- No PostgreSQL, pgrx, cargo, test, fixture, corpus, or benchmark command was
  executed by the reviewer.

## Checkpoint 2 coder evidence (2026-08-26)

- Head SHA: `255081d74aa6ce430a2a21ee5555e9569c0a0fa7`
- Timestamp: 2026-08-26T05:50:16-07:00
- Scope: canonical payload-cover descriptor and compact entry codec; exact
  row-TID/`vec_id` echo validation; conditional covered-registration identity
  while preserving no-cover V1 bytes
- Disposition: `seq01-disposition.md`

### `cargo-fmt-checkpoint2.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0; formatting clean, with only stable-rustfmt warnings for the
  repository's nightly-only import grouping configuration.

### `cargo-check-pg18-checkpoint2.log`

- Command: `cargo check --lib --no-default-features --features pg18`
- Result: exit 0; PG18 library compile completed successfully.

### `cargo-test-payload-sidecar-checkpoint2.log`

- Command: `cargo test --lib --no-default-features --features pg18 payload_sidecar::tests -- --nocapture`
- Result: exit 0; 5 passed, 0 failed, 2,592 filtered out.

### `cargo-test-registration-binding-checkpoint2.log`

- Command: `cargo test --lib --no-default-features --features pg18 registration_digest_golden_binds_private_transport_fields -- --nocapture`
- Result: exit 0; 1 passed, 0 failed, 2,596 filtered out. The test preserves
  the existing no-cover digest golden and proves an optional cover digest moves
  registration identity.

### `cargo-clippy-pg18-checkpoint2.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: expected non-zero from exactly four known main/toolchain-drift lints:
  `collapsible_if` (`ambuild.rs:139`), `unnecessary_unwrap`
  (`generation_descriptor.rs:748`), `needless_range_loop`
  (`head_sample.rs:1818`), and `items_after_test_module`
  (`remote_endpoint.rs:1052`). No Task 229 file is named.

### `clippy-inherited-files-diff-checkpoint2.log`

- Command: `git diff --exit-code origin/main -- src/am/ec_distann/ambuild.rs src/am/ec_distann/generation_descriptor.rs src/am/ec_distann/head_sample.rs src/am/ec_distann/remote_endpoint.rs`
- Result: exit 0; every strict-clippy failure file is unchanged from main.

### `cargo-clippy-pg18-checkpoint2-task-clean.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings -A clippy::collapsible_if -A clippy::unnecessary_unwrap -A clippy::needless_range_loop -A clippy::items_after_test_module`
- Result: exit 0; after allowing only the four exact inherited lint names, all
  targets are warning-clean.

No PostgreSQL, `cargo pgrx test`, fixture, corpus, or benchmark command was
run. This checkpoint makes no runtime or performance claim.

## Reviewer provenance — seq 02 (2026-08-26)

- Agent / role / model: Agent IX / reviewer / claude-opus-5
- Head SHA at review: `cd7ab0baa761ae860d76c6a092dd6455bea6b205`
- Source commit reviewed: `255081d74aa6ce430a2a21ee5555e9569c0a0fa7`
- Base SHA: `3419c9c758bea7d9940b27d9afbcf9e627e84879` (confirmed to be both the
  real `git merge-base` of HEAD and main **and** current `origin/main`, not
  just a claimed base)
- Task bucket / packet: `reviews/task-229/002-format-and-lifecycle/`
- Review scope: checkpoint 2 only — `DistannPayloadCoverDescriptorV1` canonical
  bytes/digest/schema binding, the compact null/value entry codec, TID and
  `vec_id` echo corruption checks, conditional no-cover-V1 / covered-V2
  registration identity, and the checkpoint-1 carry-in dispositions in
  `seq01-disposition.md`. Generation-descriptor V2/V3, receipt V1/V2, manifest
  V2/V3, fingerprint and lifecycle-wire dual decode, catalog OIDs, physical
  relation ownership, read path, DML, and telemetry were not required to exist
  and were not reviewed.
- Lane / fixture / storage format / rerank mode: not applicable — static code
  review of a codec/identity checkpoint; no index, corpus, or fixture was built
  or read
- Isolated vs shared surfaces: not applicable — no table or index was created
- Verdict: **DONE**; checkpoint 3 authorized (generation descriptor V2/V3 plus
  dual-version receipt/manifest/fingerprint/lifecycle-wire persistence), with
  five carry-ins recorded in `feedback/2026-08-26-02-reviewer.md` (item 1,
  covering the descriptor decode-rejection arms with tests, must be closed in
  checkpoint 3)

### `reviewer-seq02-verification.log`

- Command: read-only static inspection via `git log`, `git show`, `git diff`,
  `git merge-base`, `git rev-parse`, `grep`, `sed`, and file reads against the
  worktree at
  `/home/peter/dev/ecaz/.worktrees/task229-covering-payload-sidecar`
- Timestamp: 2026-08-26
- Result: thirteen checks X1-X13 — scope/base confirmation, descriptor
  canonical-byte unambiguity and byte-identical re-encode, new digest domain vs
  unchanged existing domains, the eleven-arm descriptor validation inventory,
  fingerprint plus per-attribute schema binding, the compact entry codec's
  encode/decode symmetry and fail-closed arms, `decode_row` TID/`vec_id` echo
  ordering, conditional V1/V2 registration bytes plus the preserved golden
  digest, the three-direction T1/T2 reloption-drift matrix, a full cross-check
  of every coder artifact against `request.md`, the checkpoint-1 carry-in
  disposition audit, the codec test-coverage inventory, and the per-row
  allocation cost of `validate()`
- Key result lines cited by the feedback file: X1 (source commit touches only
  four files under `src/am/ec_distann/`; declared base is both the real
  merge-base and current `origin/main`), X2 (every field fixed-width or
  length-prefixed, so `encode(decode(b)) == b` for every accepted `b`), X3
  (`ec_distann_payload_cover_descriptor_v1\0` is new; `BUILD_REGISTRATION_DOMAIN`
  and every other existing domain are unchanged), X4/X5 (eleven validation arms;
  the empty-collation gate cannot trip a legitimate cover because all eleven
  allowlist types have `typcollation = 0`), X7 (all three `decode_row` checks
  precede value exposure; `ItemPointer` derives `PartialEq`), X8 (no-cover bytes
  byte-identical to main, golden `c5a90122...25ab` unchanged and passing;
  `encode_registration` output is never persisted, so no registration decoder is
  owed), X9 (cover changed / removed / added between T1 and T2 all move the
  expected digest and error before `capture_source_snapshot`; `t1.rs:335` and
  `t2.rs:241` are the only `replay_registration` callers), X10 (all seven coder
  artifacts read in full and consistent with `request.md`; `src/lib.rs:57`
  `#[allow(dead_code)] mod am;` means the clean `cargo check --lib` does not
  prove the codec is wired), X12 (fourteen validator/codec rejection arms have
  no test), X13 (`validate()` costs ~5 heap allocations per attribute per call
  and runs once per encoded and per decoded row)
- No PostgreSQL, pgrx, cargo, test, fixture, corpus, or benchmark command was
  executed by the reviewer.

## Checkpoint 3 coder evidence (2026-08-26)

- Head/source SHA: `56a1b37fc632cee8a12dd3e0c32b138afdea3466`
- Base SHA: `3419c9c758bea7d9940b27d9afbcf9e627e84879`
- Timestamp: 2026-08-26T06:35:43-07:00
- Scope: generation descriptor V2/V3, Ready receipt V1/V2, epoch manifest
  V2/V3, fingerprint V2/V3, variable-length Ready-receipt-set/catalog/SQL
  consumers, frozen legacy identity fixtures, and seq-02 corruption/allocation
  carry-ins
- Lane / fixture / storage format / rerank mode: static canonical-format work;
  frozen on-disk format fixtures only; no live index, corpus, rerank mode, or
  benchmark lane
- Isolated vs shared surfaces: not applicable — no table, index, cluster, or
  benchmark surface was created
- Disposition: `seq02-disposition.md`

### `cargo-fmt-checkpoint3.log`

- Command: `cargo fmt --all -- --check`
- Result: exit 0; formatting clean. Stable rustfmt emitted only the repository's
  existing nightly-only import-grouping warnings.

### `cargo-check-pg18-checkpoint3.log`

- Command: `cargo check --lib --no-default-features --features pg18`
- Result: exit 0; PG18 library compile completed successfully in 15.42s.

### Focused unit and frozen-fixture logs

- `cargo-test-payload-sidecar-checkpoint3.log`: focused payload-sidecar suite,
  6 passed / 0 failed; includes all reviewer seq-02 corruption carry-ins.
- `cargo-test-legacy-identity-checkpoint3.log`: legacy descriptor/manifest and
  covered V3 round-trip filter, 3 passed / 0 failed (one unrelated name-match).
- `cargo-test-ready-receipt-checkpoint3.log`: V1/V2 Ready receipt identity and
  exact 303/359-byte lengths, 1 passed / 0 failed.
- `cargo-test-receipt-set-checkpoint3.log`: bounded variable V1/V2 receipt-set
  framing, 1 passed / 0 failed.
- `cargo-test-covered-identity-chain-checkpoint3.log`: covered V3 descriptor →
  V2 receipts → V3 manifest → V3 fingerprint build candidate and mismatched
  fingerprint-version rejection, 1 passed / 0 failed.
- `cargo-test-on-disk-distann-checkpoint3.log`: frozen DistANN on-disk fixture
  suite, 21 passed / 0 failed; legacy descriptor, receipt, manifest,
  fingerprint, build-candidate and receipt-set bytes re-encode identically.

Commands are the exact `cargo test` invocations recorded in each script log.
All used `--no-default-features --features pg18`; no PostgreSQL server or pgrx
fixture was started.

### Clippy and static provenance logs

- `cargo-clippy-pg18-checkpoint3.log`: strict all-target clippy reports exactly
  the four inherited Rust-1.94/main findings: `collapsible_if`
  (`ambuild.rs:139`), `unnecessary_unwrap`
  (`generation_descriptor.rs:798`, shifted from base line 748 only by inserted
  Task 229 code), `needless_range_loop` (`head_sample.rs:1818`), and
  `items_after_test_module` (`remote_endpoint.rs:1052`).
- `clippy-inherited-files-diff-checkpoint3.log`: exit 0; `ambuild.rs`,
  `head_sample.rs`, and `remote_endpoint.rs` are byte-identical to the base.
- `clippy-generation-descriptor-blame-checkpoint3.log`: the complete lint arm,
  including line 798, blames to `4fe5d5c53a` from 2026-08-01 rather than this
  checkpoint.
- `cargo-clippy-pg18-checkpoint3-task-clean.log`: exit 0; with only those four
  exact inherited lint names allowed, all targets pass under `-D warnings`.
- `git-diff-checkpoint3.log`: `git diff --check` over exact base..source exits
  0.

No live PostgreSQL, `cargo pgrx test`, corpus, benchmark, or performance command
was run. This checkpoint makes no runtime, storage, latency, or recall claim.

## Checkpoint 4 coder evidence (2026-08-27)

- Review head SHA: `c14c796b86e0c59877cc377548c36989a1a02ff3`
- Current-main merge parent: `71004a18ce770f0c17501bd3d9942742d700a6ba`
- Checkpoint source commits:
  - `134b55d66` — cataloged owner-local sidecar heap/index, handoff/seal
    population and evidence, five-OID abort/reclaim plumbing, canonical V2/V3
    fingerprint gates
  - `2cb26bfc8` — remove redundant receipt sidecar count; covered receipt is
    351 bytes and the manifest-global digest uses `owned_record_count`
  - `e4e895bff` — covered V3 publish/retain/retire/rollback/reclaim PG18 test
  - `c14c796b8` — remove the only task-local strict-clippy warning
- Task / packet: `reviews/task-229/002-format-and-lifecycle/`
- Lane / fixture / storage format / rerank mode: local PG18 single-owner
  format/lifecycle fixture; covered sidecar V1; no scan/rerank or benchmark lane
- Isolated vs shared surfaces: each `cargo pgrx test` uses its own transaction
  and unique relation stem. No benchmark table, corpus, or persistent cluster
  was created.
- Build location: the host-wide shared
  `CARGO_TARGET_DIR=/home/peter/.cargo-target`; no task-specific build graph.

### Format and compile

#### `cargo-fmt-checkpoint4.log`

- Command: `cargo fmt --all -- --check`
- Source: review head `c14c796b8`
- Result: exit 0; formatting clean, with only stable-rustfmt warnings for the
  repository's nightly-only import-grouping options.

#### `cargo-check-pg18-checkpoint4.log`

- Command: `cargo check --lib --no-default-features --features pg18`
- Source: checkpoint-4 working tree immediately before `e4e895bff`
- Result: exit 0; PG18 library compile completed in 11.98s.

### Focused Rust and frozen-format tests

- `cargo-test-payload-sidecar-checkpoint4.log`: command
  `cargo test --lib --no-default-features --features pg18
  payload_sidecar::tests -- --nocapture`; 6 passed / 0 failed.
- `cargo-test-ready-receipt-checkpoint4.log`: focused
  `ready_receipt_round_trip_verifies_digest_state_and_counts`; 1 passed / 0
  failed and validates the 351-byte covered receipt.
- `cargo-test-covered-checkpoint4.log`: `cargo test --lib
  --no-default-features --features pg18 covered -- --nocapture`; 4 passed / 0
  failed, including covered descriptor/manifest/fingerprint/candidate identity
  (and one unrelated HNSW name match).
- `cargo-test-on-disk-distann-checkpoint4.log`: `cargo test --test
  on_disk_fixtures --no-default-features --features pg18 distann_ --
  --nocapture`; 21 passed / 0 failed, preserving frozen no-cover bytes.
- Source for these four logs: `2cb26bfc8` checkpoint source; the later test-only
  commits do not change the exercised production codecs or frozen fixtures.

### Focused live PG18 lifecycle tests

#### `cargo-pgrx-test-cover-sidecar-checkpoint4.log`

- Command: `cargo pgrx test pg18 test_distann_cover_sidecar_lifecycle`
- Source: `2cb26bfc8`
- Result: exit 0; 1 passed / 0 failed in 125.89s.
- Key proof: exact begin replay preserves the sidecar pair; the heap/index
  physical shape and internal dependencies are correct; handoff writes one
  canonical row tied to graph TID/vec_id; seal exposes digest and storage
  evidence; abort drops all five generation relations.

#### `cargo-pgrx-test-cover-sidecar-participant-checkpoint4.log`

- Command: `cargo pgrx test pg18
  test_distann_cover_sidecar_retire_reclaim_rollback`
- Source: `e4e895bff`; `c14c796b8` subsequently changes only the no-cover
  fixture helper call and leaves this covered test path byte-identical.
- Result: exit 0; 1 passed / 0 failed in 78.24s after extension installation.
- Key proof: covered V3 manifest/fingerprint publication succeeds; all five
  relations remain live through publish and retirement; forced transaction
  rollback restores all five and removes its tombstone; final reclaim drops all
  five, persists `Reclaimed`, and exact replay succeeds.

### Clippy

#### `cargo-clippy-pg18-checkpoint4.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18
  -- -D warnings`
- Source: review head `c14c796b8`
- Result: expected exit 101 from exactly the four inherited main/Rust-1.94
  findings already proven by checkpoint 3: `collapsible_if`
  (`ambuild.rs:139`), `unnecessary_unwrap`
  (`generation_descriptor.rs:798`), `needless_range_loop`
  (`head_sample.rs:1818`), and `items_after_test_module`
  (`remote_endpoint.rs:1069`). The first checkpoint-4 run also found a
  task-local dead helper; `c14c796b8` fixed it, and this retained final strict
  log contains no Task 229-local finding.

#### `cargo-clippy-pg18-checkpoint4-task-clean.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18
  -- -D warnings -A clippy::collapsible_if -A clippy::unnecessary_unwrap
  -A clippy::needless_range_loop -A clippy::items_after_test_module`
- Source: review head `c14c796b8`
- Result: exit 0; all targets warning-clean after allowing only the four exact
  inherited lint names.

## Runtime-output hygiene

No custom `CARGO_TARGET_DIR`, new worktree, explicit `--run-dir`, corpus,
truth cache, benchmark suite, PostgreSQL benchmark PGDATA, or retained cluster
was created for checkpoint 4. The only live PostgreSQL surfaces were the
transaction-scoped pgrx test fixtures above.
