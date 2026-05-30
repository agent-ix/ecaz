# Suite Status

Command:

```text
/Users/peter/.cargo/bin/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite status --manifest reviews/task-68/005-zero-replica-fast-path-measurement/artifacts/suite-manifest.json
```

Result:

```text
[suite:task68-spire-zero-replica-fast-path-measurement] completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
Succeeded    precheck-host-and-tables
Succeeded    create-10k-spire-fastpath-index
Succeeded    create-100k-spire-fastpath-index
```
