# Task 107 packet 002 - AWS provisioning manifest

- Head SHA: `ef916cea7`.
- Task bucket: `reviews/task-107/002-aws-provisioning/`.
- Date: 2026-06-15.
- Purpose: provision and prepare the SPIRE AWS topology used for Task 107
  distributed benchmark work.

## Topology

- Region/AZ: `us-west-2` / `us-west-2a`.
- Coordinator: `i-0b4386fa5017f1363`, private IP `10.42.1.154`,
  `r7g.4xlarge`, 400G root volume.
- Remote node 2: `i-07bcc98c3d5d027ee`, private IP `10.42.1.60`,
  `r7g.2xlarge`, 300G root volume.
- Remote node 3: `i-00c2f2aca9dbdd6bd`, private IP `10.42.1.249`,
  `r7g.2xlarge`, 300G root volume.
- Artifact bucket: `ecaz-spire-aws-20260614203301860100000009`.
- Topology file: `aws-topology.json`.

## Coordinator Store Volumes

The coordinator has four additional 200G gp3 volumes, each mounted and exposed
as a PostgreSQL tablespace:

| Mount | Tablespace | Size |
| --- | --- | ---: |
| `/var/lib/ecaz-spire-store-1` | `ecaz_spire_store_1` | 200G |
| `/var/lib/ecaz-spire-store-2` | `ecaz_spire_store_2` | 200G |
| `/var/lib/ecaz-spire-store-3` | `ecaz_spire_store_3` | 200G |
| `/var/lib/ecaz-spire-store-4` | `ecaz_spire_store_4` | 200G |

Setup evidence:

- `setup-coordinator-store-volumes.ssm.json`
- `setup-coordinator-store-volumes.log`
- `setup-coordinator-store-volumes-rerun.log`
- `setup-coordinator-store-volumes-rerun2.log`
- `setup-coordinator-store-volumes-rerun3.log`

## Commands / Logs

- Residue cleanup:
  - `cleanup-residue-us-west-2.log`
  - `cleanup-residue-postcheck-us-west-2.log`
- Provisioning:
  - `provision.log`
  - `aws-topology.json`
- Extension packaging/upload/install:
  - `package-extension.log`
  - `source-upload.log`
  - `tarball-upload.log`
  - `bootstrap-upload.log`
  - `install-extension.log`
  - `install.log`
  - `install-i-0b4386fa5017f1363.log`
  - `install-i-07bcc98c3d5d027ee.log`
  - `install-i-00c2f2aca9dbdd6bd.log`
- Coordinator remote conninfo:
  - `install-coordinator-remote-conninfo.log`
  - `coordinator-remote-conninfo-env.redacted.log`

Large generated build tarballs, packaged extension binary directories, local
source trees, Terraform state, and TLS private material were pruned from the
packet before commit. The packet keeps operator logs and redacted/topology
metadata only.
