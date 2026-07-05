## Feedback: Scratch Socket Target Guardrails

Read `scripts/resolve_scratch_socket_dir.sh` end-to-end,
`scripts/pg17_scratch_psql.sh` and the five updated scratch
benchmark wrappers, and
`scripts/tests/test_resolve_scratch_socket_dir.py`.

### What's right

- **Precedence order is correct and minimal.**
  `TQV_PG_SOCKET_DIR` > `PGHOST` > preferred scratch socket > error.
  Explicit operator override wins, the standard libpq env wins next,
  and "guess from what's lying around on the filesystem" is demoted to
  a diagnostic. That is the only policy that lets a measurement run
  fail loudly instead of silently drifting onto `~/.pgrx`.
- **Error messages are actionable, not just terse.** Both the "refusing
  to fall back" message and the "expected socket at …" message name
  the exact env vars the caller can set and the exact path that was
  looked for. An operator hitting this mid-benchmark does not have to
  go read the script.
- **One shared helper replaces per-wrapper duplication.** Before this
  packet each scratch wrapper encoded its own fallback policy, and
  they had already drifted (the packet context calls out the
  2026-04-16 mismeasurement). Centralizing the resolution means one
  policy for every wrapper, so a future fix has one place to change.
- **`TQV_SCRATCH_TEST_ACCEPT_FILES=1` test escape hatch is
  appropriately scoped.** The helper is a `-S` check by default and
  only accepts a plain file when the test env var is set. That keeps
  the regression test hermetic without requiring the test harness to
  bind a real AF_UNIX socket, and the escape hatch is not something
  a regular caller would set by accident.
- **Python regression directly exercises the helper.** The test covers
  the three interesting states: explicit override wins, home-directory
  fallback refused, and missing preferred socket fails loudly. That's
  the right shape of coverage for a shell helper whose failure mode is
  "silently targeted the wrong cluster".

### Concerns

1. **`PGHOST` trusted unconditionally.** If `PGHOST` happens to be set
   in a shell (e.g. inherited from a previous scratch run or a user
   `.bashrc`), it takes precedence over the preferred scratch socket
   even when the scratch socket is sitting right there. That is
   arguably correct, but the mismeasurement the packet reacts to is
   exactly the class of bug this creates — env lingers in a terminal
   session. Worth a banner line or a single-line warning when
   `PGHOST` overrides an available preferred socket, so the operator
   at least sees "you are targeting PGHOST, not the scratch default".
2. **No coverage for the `PGHOST` precedence branch.** The regression
   test asserts explicit override, refused fallback, and missing
   socket — but not the "PGHOST is set, use it" branch. One extra
   case would seal the precedence contract in the test rather than
   only in the comment.
3. **Wrappers touched but no integration smoke.** The packet replaces
   the socket-selection logic inside five bench wrappers by calling
   the helper, but the validation list only includes the helper unit
   test and `bash -n` syntax checks. A single end-to-end smoke
   invocation of one representative scratch wrapper with an explicit
   `TQV_PG_SOCKET_DIR` pointing at a fake-socket fixture would prove
   the wiring, not just that the shell parses.
4. **Port resolution fixed at helper-invocation time.**
   `port="${TQV_PG_PORT:-${PGPORT:-28817}}"` is resolved once. If a
   caller sets `PGPORT` mid-script for a specific step, the helper
   won't pick that up. Probably fine given how the wrappers call it,
   but worth noting as a scope boundary.
5. **Linker gap is irrelevant here.** This packet is pure scripts +
   Python; `scripts/tests/run.sh` is the load-bearing validation and
   it passed. The packet is well-proven within its own scope.

### Observation

Small operational packet, correctly motivated. The 2026-04-16
mismeasurement is the kind of branch-level failure that justifies
spending a packet on tooling discipline — without it, every
subsequent recall/latency number on this branch would have a
"which cluster did that actually hit?" asterisk. Right work at the
right time.
