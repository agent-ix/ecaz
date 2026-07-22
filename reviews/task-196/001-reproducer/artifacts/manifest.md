# Artifact manifest — Task 196 packet 001

- Head / runner / diagnostic extension SHA:
  `62820fc767f361e922298b5ed37d2382f9741e36`
- Task bucket / packet: `reviews/task-196/001-reproducer/`
- Lane / fixture: local Intel, three independent PG18 physical owners, real
  100k staged corpus, one index per table
- Search / materialization: exact training-landmark head, RaBitQ neighbors,
  BW4/H100, eager control versus production lazy10
- Scenario: explicit `reject_multiple_windows` semantic drill after excluding
  the first 40 ranked IDs
- Profile: release PG18 plus attribution feature
- Timestamp: 2026-07-22 America/Los_Angeles

## Commands and result

The extension was built and installed with:

```text
PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18,distann-head-attribution-benchmark
```

Target and installed binaries were 24,269,976 bytes and byte-identical at
SHA-256 `b1ac1ac221c25a888e59e6cfec2ea3132ac83e65fafe3cd5d6e30304c4887e3a`.

The checked-in suite was audited and executed with:

```text
target/release/ecaz bench suite run --config reviews/task-196/001-reproducer/artifacts/task196-stable-prefix-reproducer-100k.json --artifact-dir reviews/task-196/001-reproducer/artifacts/diagnostic-run
```

The expected failed step reproduced one remote payload re-request at ranked
window 20..30. The current proven prefix contained zero duplicate ranked IDs,
but two already-materialized remote IDs shifted raw rank during deepening.
That excludes duplicate traversal output and window overlap, and identifies
rank-indexed reuse as the failed invariant.

`git diff --quiet f291bbb48adafc5840f80ed4633e8c0689e7df03..adcd95623aae91d960ff2f884ff64a95b0f6406e -- src/am/ec_distann/custom_scan.rs`
exited 0: the failing reuse implementation is unchanged from the Task 191
acceptance merge through the Task 195 parent. Task 191's semantic suite ran
only 10k, while its 100k matrix did not enable materialization correctness;
the result is therefore a Task 191 100k coverage hole, not a later CustomScan
regression.

## Files

| Artifact | SHA-256 | Purpose |
|---|---|---|
| `task196-stable-prefix-reproducer-100k.json` | `bf07d834464422ace296a18c832c9c436cc023a241d279e6e502ae9cc7ff28f3` | Checked-in real-100k suite reproducer and explicit scenario label |
| `diagnostic-run/suite-manifest.json` | `d01439807ac88450ceb4d03734138bc0be6b8f1ec91550745075404916e8068b` | Clean runner SHA, command, 2,062,449 ms duration, and expected failed step |
| `suite-run-diagnostic.log` | `2c34b986d963b1dc47317eeb93c7cd0fa2c05e08aec911899bf737d235ad272f` | Compact topology gates and attributed terminal failure |
| `diagnostic-binary-identity.log` | `466a620e39ca7b840d6c0ef9ddc67281d7427bfb590c6c2312dc8123fc9a029a` | Release target/installed byte identity |
| `diagnostic-release-install.log` | `b1f48e910b44aa974112b14321aadd4549e3d3d295eb860eb76eaa82d5af9fb8` | Release build/install transcript |
| `attribution-summary.log` | `3349c74527b8be1dccf62deec8a2a94ea34f2bf02a2ce02fd9c435ffc1669050` | Compact root-cause classification |

Node PostgreSQL logs, the full fixture transcript, single-control logs, and
other operational exhaust are deliberately not committed.
