# Task 172 / 010 artifact manifest

- Code head:
  `b85ee694f8136a19d7b481d8873f81e8f4e93983`
- Task bucket and packet: `reviews/task-172/010-macos-release-cli/`
- Branch: `task-203-ec-distann-conformance`
- Timestamp: `2026-07-30T19:24:07Z`
- Lane / fixture / storage format / rerank mode: local macOS operator-runner
  startup; no PostgreSQL fixture, index storage, or rerank mode
- Isolated vs shared surface: not applicable; no database surface was used

## Artifacts

| Artifact | Command surface | Key result |
| --- | --- | --- |
| `validation.md` | release build, release start smoke, targeted format/diff checks, and Mach-O symbol audit | release runner starts; eight retained PG18 data globals are locally defined |

No corpus, benchmark, cluster, or operational-log artifact was produced.
