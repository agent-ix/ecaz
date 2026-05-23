# Artifact Manifest — SPIRE AWS Prep Local Verification

Packet: `reviews/task-30/957-spire-aws-prep-local-verification/`
Owner: coder B (SPIRE AWS lane)
Branch: `task-30-phase13-spire-aws-prep`

## Cluster Identity

| Field       | Value (filled in as the cluster comes up)             |
| ----------- | ----------------------------------------------------- |
| Role        | SPIRE-only verification (separate from IVF work)      |
| PG version  | 18.3 (pgrx-built `~/.pgrx/18.3/pgrx-install`)         |
| PGDATA      | _TBD — `~/pg18-spire/data/`_                          |
| Port        | _TBD — `28919`_                                       |
| Socket dir  | _TBD_                                                 |
| Extensions  | ecaz (head SHA below)                                 |
| Head SHA    | _TBD on first commit_                                 |

## Artifacts

(Filled in as runs complete. Each row: artifact path, head SHA,
command, timestamp, key result line.)

| # | Artifact | Scale | Surface | Head SHA | Command | Timestamp | Key result |
|---|----------|-------|---------|----------|---------|-----------|-----------|
| _none yet_ |  |  |  |  |  |  |  |

## Snapshots

Filesystem snapshots of `$PGDATA` taken with `pg_basebackup -Ft -z`
after the cluster reaches a clean checkpoint at each scale. Stored
outside the packet when size makes inclusion impractical; the table
below records absolute path + sha256 + size.

| Scale | Snapshot path | sha256 | Size | Created |
|-------|--------------|--------|------|---------|
| 10k   | _TBD_        |        |      |         |
| 50k   | _TBD_        |        |      |         |
| 100k  | _TBD_        |        |      |         |
| 1M    | _TBD_        |        |      |         |
