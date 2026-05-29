# Artifact Manifest

Task bucket: `reviews/task-30/962-spire-phase13e-registration-sql-renderer`

Head SHA: `7bba11ff61fdb7e2999c5b204acb7dc20f1dac9e`

Timestamp: `2026-05-25T09:55:12-07:00`

## cargo-test-ecaz-cli-render-spire-registration.log

- Command: `script -q -c "cargo test -p ecaz-cli render_spire_registration" reviews/task-30/962-spire-phase13e-registration-sql-renderer/artifacts/cargo-test-ecaz-cli-render-spire-registration.log`
- Lane: local focused unit validation
- Fixture: pure `ecaz-cli` tempdir JSON fixtures
- Storage format: not applicable
- Rerank mode: not applicable
- Surface: no index/table surface; validates offline registration SQL rendering
- Result:
  - `running 5 tests`
  - `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 376 filtered out; finished in 0.00s`
