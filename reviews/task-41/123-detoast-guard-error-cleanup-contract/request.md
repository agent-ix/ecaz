# Review Request: Task 41 Invariant #2 detoast guard ERROR cleanup contract

Code commit: `b199aadbad137a5a41f36ecbbf4f175ee318a058`

## Summary

This follow-up processes reviewer feedback from packets 115 and 116.

Each local detoast guard now documents the cleanup contract explicitly:
normal Rust paths free copied detoast memory in `Drop`, while PostgreSQL
memory-context cleanup covers ERROR abort fallbacks if control does not unwind
through Rust frames.

## Scope

- Comment-only change.
- Touched only the detoast guard structs introduced or used by Task 41
  invariant #2 detoast packets.
- No guard behavior, ownership flags, pointer conversion, or error handling
  changed.

## Validation

- `cargo fmt --all --check`
- `git diff --check HEAD~1 HEAD`

No cargo check or pgrx runtime tests were run because this is a comment-only
follow-up to reviewer feedback.

## Artifacts

- `artifacts/fmt-check.log`
- `artifacts/git-diff-check.log`
- `artifacts/code-diff-stat.log`
- `artifacts/manifest.md`

## Reviewer Focus

- Confirm the comment accurately states the invariant #2 cleanup contract.
- Confirm this does not broaden the detoast refactor or overlap with the
  pending shared-helper consolidation idea.
