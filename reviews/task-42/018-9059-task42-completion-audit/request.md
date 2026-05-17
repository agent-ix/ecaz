# Review Request: Task 42 Partial Closeout Audit

## Summary

This packet records the Task 42 partial closeout audit after the PG upgrade and
WAL policy follow-up slices. Reviewer feedback narrowed the framing from
"complete" to a smoke checkpoint: the fixture, layout, matrix, qemu, WAL-policy,
and PG18 same-binary `pg_upgrade` infrastructure is present, while extensive CI
burn-in and richer live-upgrade coverage are deferred.

Original code/docs commit: `788a074a4f93b5771b21df6d720db1eb857f7066`
(`Mark Task 42 complete`). This packet has been amended by the reviewer-feedback
follow-up that downgrades the closeout to partial.

Changes:

- Marked `plan/tasks/42-on-disk-format-invariants.md` as a partial smoke
  checkpoint.
- Renamed the docs tail from remaining gaps to conditional future extensions.
- Updated `artifacts/completion-audit.md`, mapping every explicit current Task
  42 requirement to concrete evidence while flagging narrow or deferred areas.
- Linked the version-matrix framing to NFR-016-EV-3 and the WAL framing to
  ADR-070 / Task 37.

## Validation

- `cargo fmt --all -- --check`: passed with existing stable-toolchain warnings
  about unstable rustfmt options.
- `cargo test --features bench --test wal_policy`: passed (`2 passed`), with
  the existing unused-import warning in `src/am/mod.rs`.

## Reviewer Focus

- Does the audit correctly distinguish the current Task 42 smoke checkpoint
  from future conditional work that only activates when CI is steadier, new
  durable byte contracts ship, or new incompatible writable versions ship?
- Is there any current on-disk surface missing from the fixture/static/matrix
  evidence chain?
