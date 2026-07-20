# Artifact manifest

- Head SHA: `5a48c7ee93fcc2f1f201c7d2231f13cae467073e`
- Implementation commit: `5a48c7ee9` (`fix(distann): parallelize physical owner fanout`)
- Task bucket / packet: `reviews/task-179/043-parallel-physical-fanout`
- Lane: local PG18, focused static/unit/pgrx correctness validation
- Fixture: current-thread Tokio transport tests and the pgrx three-owner
  physical handoff lifecycle fixture
- Storage format / rerank mode: not applicable to this correctness packet
- Timestamp: `2026-07-12T19:09:41-07:00`

All commands run from the clean implementation head above. This packet is not
benchmark evidence and does not close the required 10k/50k/100k A/B gate.

## Validation commands and results

### PG18 clippy

```text
cargo clippy --lib --no-default-features --features pg18 -- -D warnings
```

Result: exit 0. See `clippy-pg18.log`.

### Focused transport unit tests

```text
cargo test --lib --no-default-features --features pg18 \
  'am::ec_distann::remote_transport::tests::' -- --nocapture
```

Result: exit 0; 8 passed, 0 failed. The added scheduling case drives three
100 ms owner futures on the same current-thread runtime and requires completion
under 230 ms; the suite also retains deadline, remote-error, conninfo-redaction,
order, and coverage checks. See `remote-transport-unit-pg18.log`.

### Three-owner physical handoff regression

```text
cargo pgrx test pg18 test_distann_three_owner_physical_handoff
```

Result: exit 0; 1 passed, 0 failed. See
`three-owner-physical-handoff-pg18.log`.

The pgrx “running for over 60 seconds” line includes extension compilation,
installation, and SQL generation. It is not owner RPC or lifecycle execution
time.

## Artifact index

- `clippy-pg18.log`: warnings-denied PG18 library lint.
- `remote-transport-unit-pg18.log`: deadline, error, conninfo redaction,
  ordering/coverage, and concurrent-owner scheduling tests.
- `three-owner-physical-handoff-pg18.log`: physical three-owner lifecycle
  regression.

No PostgreSQL server log, corpus, or operational polling output is committed.
