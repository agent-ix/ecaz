# Artifact Manifest: 31017 SPIRE Phase 12b Audit Decisions

Head SHA: `05598eb538bd7883b845596e6716205a0e94e1f3`
Packet/topic: `31017-spire-phase12b-audit-decisions`
Timestamp: `2026-05-13T17:35:14-07:00`
Lane: Phase 12b cleanup, midphase audit follow-up
Fixture: not applicable
Storage format: not applicable
Rerank mode: not applicable
Surface isolation: not a measurement run

## Artifacts

### `cargo-fmt-check.log`

Command:

```sh
cargo fmt --check
```

Key result:

```text
Script done on 2026-05-13 17:35:05-07:00 [COMMAND_EXIT_CODE="0"]
```

Notes: stable rustfmt emitted the repository's existing unstable-option
warnings for `imports_granularity` and `group_imports`.

### `git-diff-check.log`

Command:

```sh
git diff --check
```

Key result:

```text
Script done on 2026-05-13 17:35:04-07:00 [COMMAND_EXIT_CODE="0"]
```

### `line-counts.log`

Command:

```sh
wc -l src/tests/mod.rs src/tests/remote_search.rs src/tests/dml_frontdoor.rs src/lib.rs src/am/ec_spire/custom_scan/explain.rs
```

Key result:

```text
  36197 src/tests/mod.rs
   2634 src/tests/remote_search.rs
   2562 src/tests/dml_frontdoor.rs
  17812 src/lib.rs
     80 src/am/ec_spire/custom_scan/explain.rs
  59285 total
```

### `decision-location-check.log`

Command:

```sh
rg -n 'Midphase audit decision|tuple_transport_status|stable shape marker|src/tests/.*2,500|src/lib.rs.*fixture' plan/tasks/task30-phase12b-spire-cleanup.md src/am/ec_spire/custom_scan/explain.rs
```

Key result:

```text
plan/tasks/task30-phase12b-spire-cleanup.md
112:- Midphase audit decision in packet `31017`: the hard 2,500-line cap in
119:- Midphase audit decision in packet `31017`: the original
121:  requires `src/lib.rs` to contain no `test_ec_spire_*` fixture bodies
390:- `src/lib.rs` contains no `test_ec_spire_*` fixture bodies. It may

src/am/ec_spire/custom_scan/explain.rs
33:        // Minimal Phase 12b contract: this is a stable shape marker, not a
35:        pg_sys::ExplainPropertyText(c"tuple_transport_status".as_ptr(), c"ready".as_ptr(), es);
```
