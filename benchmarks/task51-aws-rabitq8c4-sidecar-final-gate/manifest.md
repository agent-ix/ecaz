# Task 51 AWS RaBitQ8 Sidecar Final Gate Attempt

- Timestamp: 2026-05-23T23:34:49Z
- Branch: `aws-optimization-ivf-rabitq-spire`
- Head SHA after cloud bench fix: `19b410928c131530063c5391147fb0bc387c8b4a`
- Remote branch installed for the AWS attempt: `aws-optimization-ivf-rabitq-spire` at the pre-fix branch head
- Task bucket: `reviews/task-51`
- Benchmark packet: `benchmarks/task51-aws-rabitq8c4-sidecar-final-gate`
- Scope: IVF/RaBitQ sidecar variants only (`rabitq8`, `rabitq8ls`, `rabitq8c3`, `rabitq8c4`)
- Excluded: vchord, pgvectorscale/DiskANN, unchanged comparator reruns
- AWS profile: `10k-medium`
- AWS shape: DB `m8g.xlarge`, loader `c8g.medium`
- Restored snapshot: `snap-0758119609e81ab7f`
- Shutdown snapshot: `snap-0b72153293b0b749b`
- Final cloud status: `state: down`, `$0.00/hr running`, retained storage only

## Outcome

No AWS sidecar benchmark numbers were produced in this attempt.

The first `ecaz cloud bench` attempt failed before measurement because the existing implementation ran `ecaz bench suite` locally against the private DB IP `10.42.1.122`; local `psql` timed out before the precheck could complete.

The cloud bench command was then patched locally to run the suite on the DB host over `/var/run/postgresql` through the existing `ecaz` SSM wrapper, upload artifacts to S3, and sync them back. That remote attempt started SSM command `897e691d-cd68-4837-bcf1-b0d9cea44ccd`, but did not complete inside the bounded wait. Compute was stopped before any sidecar results were returned.

Do not use this packet to claim `rabitq8c4` AWS speed or recall. It is a failed/incomplete AWS final gate with local-only recall evidence still coming from `benchmarks/task51-local-rabitq8-sidecar-recall-sweep`.

## Commands And Artifacts

- `suite.json`: SuiteConfig for the intended sidecar-only AWS sweep.
- `artifacts/suite-audit-full-sidecar-local.log`: `ecaz bench suite audit` passed for the sidecar-only suite.
- `artifacts/suite-dry-run-full-sidecar-local.log`: dry-run expansion showing the intended precheck plus sidecar-rerank command.
- `artifacts/suite-dry-run-full-sidecar-manifest.json`: dry-run manifest for the sidecar-only suite.
- `artifacts/cloud-up-from-snapshot.log`: stack restore attempt from `snap-0758119609e81ab7f`; stdout was also observed in terminal.
- `artifacts/cloud-install.log`: branch install returned `install: profile=10k-medium db=10.42.1.122 ref=aws-optimization-ivf-rabitq-spire ok`; the log file itself is empty because this command's stdout was not mirrored.
- `artifacts/cloud-bench-full-sidecar.log`: original local-private-IP `cloud bench` failure.
- `artifacts/precheck-preserved-1m-ivf-rabitq.log`: `psql` timeout from the original local-private-IP precheck.
- `artifacts/suite-manifest.json`: failed original suite manifest; precheck failed and sidecar step remained pending.
- `artifacts/cloud-bench-remote-full-sidecar.log`: patched remote-host run failed after teardown, reporting SSM command `897e691d-cd68-4837-bcf1-b0d9cea44ccd` as failed.
- `artifacts/cloud-snapshot-after-stalled-sidecar.log`: snapshot command returned `snap-0b72153293b0b749b`; the log file itself is empty because stdout was not mirrored.
- `artifacts/cloud-down-after-stalled-sidecar.log`: destroy failed after compute teardown because the S3 bucket was non-empty and EBS volume deletion timed out.
- `artifacts/cloud-status-final.log`: final status command returned `state: down`, snapshot `snap-0b72153293b0b749b`, `$0.00/hr running`; the log file itself is empty because stdout was not mirrored.

## Key Lines

```text
[suite:task51-aws-rabitq8-sidecar-final-gate] audit passed: 2 steps
psql: error: connection to server at "10.42.1.122", port 5432 failed: Connection timed out
ssm command 897e691d-cd68-4837-bcf1-b0d9cea44ccd on i-0b3375453f169ab75 ended in Failed (rc=-1)
profile:  10k-medium
state:    down
snapshot: snap-0b72153293b0b749b
cost:     ~$0.00/hr running, ~$4.00/mo retained storage
```
