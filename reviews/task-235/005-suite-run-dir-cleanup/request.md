---
task: 235
packet: 005-suite-run-dir-cleanup
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 01
---

# Task 235 DistANN suite run-directory cleanup

Please review safety checkpoint
`dc3ddbae5e4b4f6f49c12299670da076f20d4b6b`.

This checkpoint fixes the runner-side cause of accumulated Task 167/235
fixtures before the remaining Task 235 candidate measurements run. It changes
no extension or index behavior and creates no benchmark result.

`ecaz bench suite` now treats every selected terminal
`distann-local-multinode` run directory as suite-owned temporary state. After
result extraction, `results.jsonl`, threshold evaluation, and durable expected-
artifact verification, it removes each unique stopped fixture directory and
records `run_dir_cleanup=removed` in the final suite manifest. Cleanup also
runs before returning an ordinary step or threshold failure.

The cleanup fails closed rather than deleting when expected packet artifacts
are missing, a `postmaster.pid` remains, the path is not a strict child of
`ECAZ_CLUSTER_ROOT`, or the path resolves inside the repository or Cargo target
directory. Parent-directory components and symlink escapes are rejected. A
fixture may be retained only by setting a non-empty single-line
`retain_run_dir_reason`; the reason and `run_dir_cleanup=retained` are written
to the manifest.

Suite execution also rejects an explicitly configured repository-local
`CARGO_TARGET_DIR`. This host continues to use the inherited shared
`/home/peter/.cargo-target`; no task-specific target directory is introduced.

Validation is limited to one shared-target `cargo check -p ecaz-cli` and two
pure path-policy unit tests. No PostgreSQL node, PGDATA directory, corpus, or
benchmark fixture was created for this packet.

Please focus on deletion target resolution, artifact-before-cleanup ordering,
postmaster fail-closed behavior, shared-run-dir grouping, retained-fixture
provenance, and whether deferring the normal step error until cleanup preserves
the runner's failure semantics.
