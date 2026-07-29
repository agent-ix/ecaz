# Resource checks for efficient rerun

- Date: 2026-07-27
- Check command: `free -h; df -h .; ps -eo pid,etime,pcpu,pmem,rss,stat,cmd`
- Pre-run: 59 GiB available memory; 634 GiB available disk.
- During graph setup: the largest observed active setup backend was about
  3.1 GiB RSS; host availability remained at least 56 GiB.
- Post-run: 59 GiB available memory; 628 GiB available disk; no current-run
  PostgreSQL or `ecaz` processes remained.

These are operator resource observations, not benchmark result metrics. The
completed run did not reproduce the prior 52 GiB latency-backend growth.
