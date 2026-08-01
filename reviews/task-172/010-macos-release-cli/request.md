---
task: 172
packet: 010-macos-release-cli
role: coder
status: review-requested
head: b85ee694f8136a19d7b481d8873f81e8f4e93983
date: 2026-07-30
---

# Review request: make the macOS release runner launchable

## Requested decision

Please review commit `b85ee694f8136a19d7b481d8873f81e8f4e93983`,
which removes a local-host blocker for Task 172: the fat-LTO release
`ecaz` CLI now reaches `main` on macOS instead of aborting in dyld on an
unresolved PostgreSQL backend global.

This is a runner-capability checkpoint. It does not run the physical matrix or
claim Task 172 complete.

## Scope

The CLI statically links `ecaz::bench_api`, whose unreachable link graph
references PostgreSQL backend data globals. The existing macOS-only stub module
already defines the globals retained by earlier builds. Current fat LTO retained
eight additional PG18 globals:

- `MyDatabaseId`, `MyProcPid`, and `MyProcNumber`;
- `ProcGlobal`;
- `InterruptPending`, `QueryCancelPending`, and `ProcDiePending`; and
- `XactIsoLevel`.

Their inert definitions use the exact storage types declared by the installed
PG18 headers. The CLI never calls the PostgreSQL-backend paths; callable
PostgreSQL functions remain dynamically looked up and are not stubbed.

The Apple-silicon build-matrix lane now builds the release profile and launches
`ecaz bench suite --help`. A compile-only or dev-profile check would not catch
this defect because release fat LTO retains a larger extension graph.

## Validation

See `artifacts/validation.md` and `artifacts/manifest.md`.

- Pre-fix release start: exit 134, `_MyDatabaseId` not found.
- `cargo build --release -p ecaz-cli`: pass.
- Release `ecaz bench suite --help`: pass, exit 0.
- Required absolute operator binary installed from the committed release build;
  its SHA-256 matches `target/release/ecaz` and its suite help starts.
- Undefined-data audit: none of the eight PG18 globals remains undefined; all
  eight are defined by the executable.
- Targeted Rust formatting and `git diff --check`: pass.

No PostgreSQL test or benchmark was run. The change affects process startup
only, and the release build/start smoke is the narrow validation for that
behavior.

## Reviewer focus

1. Confirm the dummy definitions match the PG18 header storage types.
2. Confirm limiting them to macOS and to data globals preserves the boundary
   that the CLI must never invoke extension backend code.
3. Confirm the release-profile launch smoke is the right regression check.

This request remains open for outside review.
