# Packet 016 — P1 remediation (decide/abort hole + privilege gap)

Task bucket: `reviews/task-179/`; packet `016-p1-remediation/`.
Surface: `ec_distann_decide_epoch_publish` / `recover_epoch_publish` /
`abort_epoch_build` (`build_coordinator.rs`) + privilege block (`lib.rs`).

## Commit under review
- `1311d10cf9eca4669a57c3f8b666f0b790f93b90` — fix(distann): close decide/abort state hole + privilege gap (packet 012 P1s).

## Artifacts
| File | Head SHA | Command | Key result |
|---|---|---|---|
| `pgrx-decide-abort-guards.log` | `1311d10cf9eca4669a57c3f8b666f0b790f93b90` | `cargo pgrx test pg18 --no-default-features --features pg18 test_distann_decide_abort_guards` | `test result: ok. 1 passed` — decide-after-abort fails EC_EPOCH_STATE; abort-after-decide fails EC_BUILD_STATE, decision stays Pending / registration Decided (generation not destroyed); decided build recovers to Published |

Also re-verified at `1311d10cf9eca4669a57c3f8b666f0b790f93b90`: test_distann_build_epoch_single_node and
test_distann_multi_epoch_publish pass under the Ready->Decided->Published machine.
`cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) pass.
