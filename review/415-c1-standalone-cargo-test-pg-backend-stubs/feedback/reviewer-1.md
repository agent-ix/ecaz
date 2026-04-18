## Feedback: Standalone Cargo Test PG Backend Stubs

Read `build.rs`, `csrc/standalone_pg_backend_stubs.c` (240 lines),
`src/standalone_pg_backend_stubs.rs`, the updated `Cargo.toml`
`.cargo/config.toml`, and `scripts/run_pgrx_pg17_test.sh`.

### What's right

- **This is the load-bearing packet of the whole 378–418 arc.**
  Every prior packet on this branch that added a `#[pg_test]`
  carried "cargo test fails with unresolved PostgreSQL symbols"
  as boilerplate. This packet actually fixes the symptom at
  source: a 240-line C shim for `CurrentMemoryContext`,
  `PG_exception_stack`, `error_context_stack`,
  `errstart`/`errfinish`, `CopyErrorData`, etc., linked only for
  the test lane. That turns `cargo test` from aspirational into
  a real checkpoint.
- **Correctly scoped to Linux/x86_64 only.** `build.rs`
  guards `target_os == "linux" && target_arch == "x86_64"`.
  Doesn't pretend to fix other platforms, doesn't silently link
  stubs into production. The `#[cfg(all(test, target_arch =
  "x86_64", target_os = "linux"))]` gate on the Rust side is the
  mirror-image gate — both halves must agree before stubs link.
- **Backend errors routed back into Rust panics.**
  `tqvector_test_pg_backend_panic(...)` bridges the C
  `ereport`-style error flow into panic termination, so a test
  that hits a backend error path still fails visibly instead of
  silently returning a zeroed struct. That is the difference
  between "linker is satisfied" and "the test lane is real."
- **Wrapper simplified, not inflated.** `run_pgrx_pg17_test.sh`
  became a direct `cargo pgrx test pg17 "$@"` passthrough. Packet
  explicitly calls out "the wrapper no longer masks what the
  underlying lane is doing" — exactly the right framing. Hiding
  the pgrx install-destination problem behind wrapper logic would
  have been the wrong fix.
- **Honest about what this *doesn't* fix.** `bash
  scripts/run_pgrx_pg17_test.sh` still fails — now on a
  read-only `/home/peter/.pgrx` install destination, not on the
  linker. That is a sandbox/filesystem problem, not a code
  problem. Separating those two failure modes is the win.

### Concerns

1. **`cargo test` passing is the claim; the packet does not name
   which tests it actually exercised.** The packet says `cargo
   test` now passes, but doesn't list *how many* tests ran, or
   whether the specific `#[pg_test]`s added across 378–418 (the
   round-trip, REINDEX-guardrail, rerank-parity, binary-fixture
   tests) are now among them. A `cargo test 2>&1 | tail -20`
   excerpt would convert the claim from "it passes" to "N tests
   ran, K of which are the ones previously blocked." Without it,
   the question "did the tests from packets 393, 403, 404, 409,
   411 now actually run?" is still open.
2. **Stub fidelity is not specified.** 240 lines of backend stubs
   is enough surface that fidelity matters. Some tests may drive
   paths that hit backend globals in ways the stub returns
   default-zero for, producing a green test that proves nothing.
   Worth a short section naming which stub returns are faithful
   (e.g., `CurrentMemoryContext` allocation) vs which are
   stubbed-but-minimal (e.g., `errstart` always signals panic).
   Otherwise "cargo test green" could paper over a test that
   exercises a stubbed boundary.
3. **No CI plan named.** This packet unblocks local `cargo test`,
   but the arc's bigger problem is "no GitHub CI at all." The
   packet is the technical prerequisite to standing up CI
   (because the standalone stubs are exactly what CI would need
   to run `cargo test`), but it does not name that follow-up.
   Natural next packet: wire `cargo test` into a GitHub Actions
   workflow on Linux/x86_64 so the stubs get exercised on every
   PR.
4. **`run_pgrx_pg17_test.sh` read-only failure is real but
   in-scope.** The pgrx-install sandbox problem remains. Not a
   concern with this packet, but the merge reviewer should know
   that `cargo pgrx test pg17` is still dark here, and the
   `#[pg_test]`s specifically marked `pg_test` (vs plain `#[test]`)
   may still need pgrx to exercise. Worth one sentence
   distinguishing "which tests the stubs cover" from "which tests
   still need real pgrx."

### Observation

This is the packet that answers "can the 378–418 test surface
ever actually run on this workstation." For most of the arc the
answer was "no, cargo pgrx test pg17 is linker-blocked." This
packet converts that to "yes for `#[test]`, and the remaining
`#[pg_test]` gap is a sandbox filesystem issue, not a code
problem." Merge-critical. The one piece missing before merge is
a captured `cargo test` run that names the specific 378–418 tests
now green — without that, the landing proof still rests on
`cargo check`.
