## Feedback: Scratch Socket Override Visibility

Read the updated `scripts/resolve_scratch_socket_dir.sh`, the new
`scripts/tests/test_pg17_scratch_psql_socket_resolution.py`,
the expanded `scripts/tests/test_resolve_scratch_socket_dir.py`,
and the inline Rust-constant pointer added to
`scripts/restart_adr030_scratch.sh`.

### What's right

- **Directly addresses every nit raised on `401` / `402`.** The
  `PGHOST` override now warns on stderr when it wins while the
  preferred scratch socket is also present, the regression suite
  now covers the `PGHOST` precedence branch explicitly, one real
  launcher now has a fake-`psql` smoke test, and the restart
  wrapper defaults point back at the Rust constants they mirror.
  Four concerns, four targeted fixes — no scope creep.
- **Tempdir-backed hermetic fixtures replace the test-only file
  override.** The earlier `TQV_SCRATCH_TEST_ACCEPT_FILES=1` escape
  hatch is gone in favor of `TQV_SCRATCH_TEST_PREFERRED_SOCKET_DIR`
  / `TQV_SCRATCH_TEST_FALLBACK_SOCKET_DIR`. That is a cleaner
  seam — tests now control *where* the helper looks rather than
  *whether* it accepts files, so the production socket-vs-file
  distinction is preserved in code while fixtures stay fully
  hermetic.
- **End-to-end smoke through a fake `psql` is the right shape of
  test.** `test_pg17_scratch_psql_socket_resolution.py` drives the
  real wrapper against a fake `psql` binary and asserts the
  resolved socket is threaded through. That is the test that
  catches "helper is correct, but the wrapper forgot to pass
  `-h`" — a class of bug the helper unit tests literally cannot
  see.
- **`PGHOST` warning chosen over rejection.** The packet keeps
  `PGHOST` precedence intact and only adds a stderr warning when
  it would silently win. That is the right tradeoff: existing
  tooling that legitimately sets `PGHOST` keeps working, but a
  measurement run against the wrong cluster now leaves a trail.
- **Rust-default pointer comment is tiny and correct.** The
  dependency from the shell defaults onto
  `PQ_FASTSCAN_DEFAULT_LIVE_RERANK_WINDOW` /
  `PQ_FASTSCAN_DEFAULT_TRAVERSAL_SCORE_MODE_NAME` is now explicit
  in the wrapper, so a future `401`-style default flip has a
  visible pointer to update both places.

### Concerns

1. **Warning is stderr-only, not persisted.** If an operator
   pipes the wrapper's stdout into a log and discards stderr
   (common in CI), the `PGHOST` override warning is invisible.
   Optional follow-up: also emit the warning into the wrapper's
   banner on stdout when `--verbose` is set, or unconditionally
   for non-TTY outputs. Out of scope here, but worth a note.
2. **Smoke coverage is one wrapper deep.**
   `pg17_scratch_psql.sh` is covered; the five bench wrappers
   routed through the helper in packet `402` are not. One of them
   could drop the `-h` flag in a future refactor and only the
   bench lane would notice. A parametrized variant of the smoke
   test that iterates over all the scratch wrappers would close
   that gap with very little extra code.
3. **`bench_sql_latency_verified_scratch.sh` not listed in the
   `bash -n` validation.** The packet lists three scripts under
   `bash -n` but the broader scratch-wrapper family touched by
   `402` is not re-validated here. Not load-bearing — this packet
   didn't modify them — but a one-line expansion keeps the habit.
4. **Linker gap still relevant only by reflex.** This packet is
   scripts-only; the `cargo pgrx test pg17` note at the bottom is
   boilerplate, not load-bearing. The real validation
   (`scripts/tests/run.sh`) passed, which is the right bar for a
   shell-helper packet. Worth trimming the boilerplate in
   scripts-only packets so it doesn't dilute the actual signal.

### Observation

Small, well-scoped reviewer-response packet. It does exactly what
a follow-up should: each named concern gets a visible fix, and the
fixes don't reopen the wider design. The tempdir-backed hermetic
fixtures are a genuine improvement on the earlier test escape
hatch — this packet shipped a better test seam as a side effect of
responding to feedback, which is the shape of work you want more
of.
