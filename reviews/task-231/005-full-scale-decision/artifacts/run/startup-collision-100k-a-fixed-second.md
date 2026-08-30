# Warm-100k pair A fixed-second startup collision

- Timestamp: `2026-08-30T06:28:54-07:00`.
- Extension SHA: `66b53998a955b583ca43c0e967806aa29e0a4404`;
  profile `release`.
- The fresh warm-100k pair A control completed successfully. The following
  fixed-stride arm failed before PostgreSQL node 1 started and before any
  measurement was taken.
- `startup-collision-node1-postgres.log` reports that `127.0.0.1:46490` was
  already in use and PostgreSQL could not create a TCP listen socket.
- Immediate read-only inspection found no remaining listener on `46490` and
  no process associated with the task run directory. Ports `46490`, `46491`,
  and `46492` were all clear before retry. The collision was transient; it was
  not a storage-capacity failure (filesystem 76% used, 234 GiB available) and
  not an extension or benchmark gate failure.
- The runner marked only
  `task231-warm-100k-a-fixed-second` failed; the preceding precheck and nine
  measurement steps remained succeeded. `apply_resume` reuses only succeeded
  steps, so the failed step and all pending steps rerun.
- Before cleanup, `pg_ctl status` reported no server running for all three
  partial node directories. The incomplete 117 MiB task-owned run directory
  `/home/peter/.ecaz/clusters/task231-warm-100k-a-fixed-second` was then
  removed. It was not evidence and is not recoverable; all failure receipts
  were preserved first.
- Archived retry boundary artifacts:
  - `suite-manifest-startup-collision.json` SHA-256
    `97a455941b53b15792d3f997b2618a7dc2f803fc359ad93fd8e21777444217d2`.
  - `results-startup-collision.jsonl` SHA-256
    `994ddebe5e5d24d6a9c5a25f86b850293c09822338cdb70df918e69f2a32301e`.
  - `suite-run-startup-collision.log` SHA-256
    `04c4d7960e5f70534190c9fa4371a4f4f5b5c414053f00c75b58cc779252e704`.
  - `warm/100k/pair-a/fixed-second/startup-collision-distann-local-multinode.log`
    SHA-256
    `d29130a802b2bbbe1a42587a1f8bc5006812a33e4972367f5ad96b008e5dffa1`.
  - `warm/100k/pair-a/fixed-second/startup-collision-node1-postgres.log`
    SHA-256
    `b2afa26322b353cc2b5e07bb188a54048021c1e4a226fbbeee7134205b655c50`.
- Disposition: operational startup collision, not A/B evidence. Resume the
  failed/pending suffix from `suite-manifest-startup-collision.json`; do not
  rerun or replace any succeeded arm.
