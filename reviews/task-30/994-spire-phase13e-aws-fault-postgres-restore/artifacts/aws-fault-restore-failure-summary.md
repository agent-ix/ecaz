# AWS Fault Restore Failure Summary

Prior packet evidence:

- Packet: `reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling`
- Run artifact dir: `reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/aws-correctness-after-tunnel-restart-fix`
- Command: `SPIRE_AWS_ALLOW_PREEXISTING_RESIDUE=1 make -C infra/spire-aws pass-correctness ARTIFACT_DIR=reviews/task-30/993-spire-phase13e-aws-correctness-after-pooling/artifacts/aws-correctness-after-tunnel-restart-fix`

Observed behavior:

- Synthetic remote placement and production-read profile completed on Graviton.
- Degraded fault drill stopped remote node 2 and returned remote heap candidates from the surviving remotes.
- The harness restarted the stopped EC2 instance and restarted the local SSM tunnel.
- Remote node 2 SQL did not become ready within 300s.
- The failing command was the restore-to-strict publication against `127.0.0.1:15433`.

Representative failure line:

```text
remote node 2 SQL did not become ready within 300s
psql: error: connection to server at "127.0.0.1", port 15433 failed: Connection refused
```

Interpretation:

This is an AWS lifecycle harness gap: after EC2 stop/start, SSM and the port forward returned, but PostgreSQL was not accepting connections on the restored node. The SPIRE scan path had already returned remote data before this restore step.
