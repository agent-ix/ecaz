# Task 43 Review Request: Cargo-Careful Mirroring

## Scope

This packet closes G6 in the Task 43 campaign tracker: cargo-careful mirrors
every currently path-liftable Miri surface, and non-mirrored SPIRE surfaces now
have explicit blockers and extraction plans instead of vague deferrals.

Code/doc changes:

- Updated `docs/hardening.md` from 67 to 69 cargo-careful harness tests.
- Updated `reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`:
  - G6 is now `Done`.
  - `012-careful-mirroring` is now `Done`.
  - SPIRE top-k, routing, vacuum/delete-delta, remote typed payload, and
    serialization cargo-careful mirrors are explicitly blocked with concrete
    extraction or micro-harness requirements.

No production Rust code was changed in this packet.

## Evidence

- `artifacts/careful-harness-cargo-test.log`
  - `cargo test --manifest-path hardening/careful/Cargo.toml --lib`
  - Result: `69 passed; 0 failed`.
- `artifacts/make-careful.log`
  - `make careful`
  - Result: `69 passed; 0 failed`; doctests `0 passed; 0 failed`.
- `artifacts/cargo-fmt-check.log`
  - `cargo fmt --all -- --check`
  - Result: exit 0.
- `artifacts/git-diff-check.log`
  - `git diff --check`
  - Result: exit 0.
- `artifacts/manifest.md`
  - Records commands, head SHA, path-lifted modules, and SPIRE blockers.

## Review Focus

- Check that the G6 tracker state is honest: path-lifted storage, DiskANN, and
  HNSW are mirrored; SPIRE mirrors are blocked with concrete required work.
- Check that the SPIRE blocker language is specific enough to guide a future
  SPIRE careful micro-harness or pgrx-free extraction.
- Check that no completion claim is made for G7 mutation probes or G8 final
  audit.
