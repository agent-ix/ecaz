# Timed-Out Attempt: phase1-rabitq-1m-l1-control

Status: failed during index build due to SSM RunShellScript execution timeout.

## What Happened

Command `cedefb52-a83b-4cba-a63b-84e7fcf516fa` used a high
`send-command --timeout-seconds 172800`, but the `AWS-RunShellScript` document
also has an `executionTimeout` parameter. Because the first payload omitted
that parameter, SSM killed the plugin after the default 3600 seconds.

The command had already completed the 1m corpus download and load/encode work:

- corpus rows: 990,000;
- query rows: 10,000;
- corpus copy: 317.26s;
- encode: 433.28s;
- query copy: 3.38s;
- killed while building `task107_phase1_rabitq_1m_l1_idx`.

## Post-Timeout State

Read-only SQL checks after the timeout showed no active CREATE INDEX backend
and a valid completed index:

- `task107_phase1_rabitq_1m_l1_corpus`: 990,000 rows;
- `task107_phase1_rabitq_1m_l1_queries`: 10,000 rows;
- `task107_phase1_rabitq_1m_l1_idx`: `indisready=true`,
  `indisvalid=true`, `indislive=true`, `amname=ec_spire`;
- reloptions:
  `{local_store_count=1,local_store_tablespaces=ecaz_spire_store_1,storage_format=rabitq}`.

## Recovery

The original payload now records `executionTimeout=172800` so future full
cell runs do not inherit the document default. Recovery uses
`ssm-resume-parameters.json`, which resumes from the valid loaded/indexed
state, runs inspect, runs the packet-local `ecaz bench suite`, captures storage
through the suite, and then cleans up only `task107_phase1_rabitq_1m_l1%`.
