# Review Request: Parallel AWS Node Install

## Summary

This checkpoint removes the serial node-bootstrap bottleneck from the established SPIRE AWS install harness.

The previous representative run showed Terraform/EC2 was fast, but node install spent roughly 18-20 minutes per node because `install.sh` submitted and waited for each node one at a time. The script header already described parallel SSM install behavior; the implementation now matches that contract:

- split SSM install into `start_install_command` and `wait_install_command`
- submit coordinator + all remote install commands first
- wait for all submitted commands afterward, preserving per-node logs
- configure coordinator remote conninfo only after all node installs complete
- add `scripts/spire-aws/check-install-parallel-local.sh`, a local mocked self-check that proves all install sends occur before install waits without touching AWS

## Evidence

- `artifacts/bash-n.log`: syntax validation for `install.sh` and the new self-check script.
- `artifacts/install-parallel-selfcheck.log`: mocked local install run showing four install sends before any install wait, then coordinator conninfo.

Key self-check sequence:

```text
install-send i-coord
install-send i-remote-1
install-send i-remote-2
install-send i-remote-3
install-wait i-coord
...
SPIRE AWS install parallel self-check passed: install_sends=4 install_waits=4
```

## Notes

No AWS resources were started for this checkpoint. This is a setup-churn reduction before the next representative Graviton run.
