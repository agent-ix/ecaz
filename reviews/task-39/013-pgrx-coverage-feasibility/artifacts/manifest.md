# Task 39 pgrx Coverage Feasibility Artifacts

- Head SHA at first probe run: `2b67adc0e2c43d617e873c728b02fc93a1e81f30`
- Implementation commit under review: `b56b386f515a0f32e6bb063b9d9ef8c47024e1d7`
- Task bucket: `reviews/task-39/013-pgrx-coverage-feasibility`
- Timestamp: `2026-05-19T00:55:24Z`
- Lane: Task 39 pgrx coverage feasibility probe
- Fixture/storage/rerank: not applicable; PG18 pgrx test instrumentation probe
- Index/table isolation: not reached; probe aborted before live backend tests

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `pgrx-coverage-probe.log` | `script -q reviews/task-39/013-pgrx-coverage-feasibility/artifacts/pgrx-coverage-probe.log env RUSTFLAGS='-C target-cpu=native -C link-arg=-undefined -C link-arg=dynamic_lookup -C instrument-coverage' LLVM_PROFILE_FILE='target/quality/pgrx-coverage-probe/profraw/ecaz-%p-%m.profraw' cargo pgrx test pg18` | Failed before live backend tests; relative profile path errors; lib test binary aborted on `_BufferBlocks`. |
| `pgrx-coverage-absolute-profile-probe.log` | `script -q reviews/task-39/013-pgrx-coverage-feasibility/artifacts/pgrx-coverage-absolute-profile-probe.log env RUSTFLAGS='-C target-cpu=native -C link-arg=-undefined -C link-arg=dynamic_lookup -C instrument-coverage' LLVM_PROFILE_FILE='/Users/peter/dev/tqvector/target/quality/pgrx-coverage-probe/profraw-absolute/ecaz-%p-%m.profraw' cargo pgrx test pg18` | Failed before live backend tests; same `_BufferBlocks` abort. |
| `pgrx-coverage-probe-key-lines.txt` | `rg -n "LLVM Profile Error|dyld|symbol not found|Finished \`test\` profile|Running unittests" pgrx-coverage-probe.log` | Extracts profile errors and loader abort from the relative-path probe. |
| `pgrx-coverage-absolute-profile-key-lines.txt` | `rg -n "LLVM Profile Error|dyld|symbol not found|Finished \`test\` profile|Running unittests" pgrx-coverage-absolute-profile-probe.log` | Extracts loader abort from the absolute-path rerun. |
| `profraw-relative-files.txt` | `find target/quality/pgrx-coverage-probe/profraw -type f -print | sort` | 149 profile files emitted before abort. |
| `profraw-absolute-files.txt` | `find target/quality/pgrx-coverage-probe/profraw-absolute -type f -print | sort` | 0 profile files emitted by the rerun before abort. |
| `profraw-counts.txt` | `wc -l profraw-relative-files.txt profraw-absolute-files.txt` | Records the profile-file counts above. |
| `git-diff-check.log` | `script -q reviews/task-39/013-pgrx-coverage-feasibility/artifacts/git-diff-check.log git diff --check` | Clean. |

## Cited Lines

```text
LLVM Profile Error: Failed to write file "target/quality/pgrx-coverage-probe/profraw/..."
dyld[37721]: symbol not found in flat namespace '_BufferBlocks'
dyld[37774]: symbol not found in flat namespace '_BufferBlocks'
149 reviews/task-39/013-pgrx-coverage-feasibility/artifacts/profraw-relative-files.txt
0 reviews/task-39/013-pgrx-coverage-feasibility/artifacts/profraw-absolute-files.txt
```
