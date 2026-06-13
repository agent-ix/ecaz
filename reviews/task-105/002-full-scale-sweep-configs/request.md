# Review request — Task 105 packet 002: full-scale sweep configs

- Task: 105, Phase 2 prep
- Coder: Task 99/102/103 author lane
- Date: 2026-06-12

`artifacts/gen_t105_sweep.py` (source of truth) emits, all dry-run
clean:

- `t105-fixtures-{10k,50k,1m}.sql` — per-scale per-variant fixture
  replication from the snapshot sources (`real_10k_ivf_tq`,
  `real_50k_ivf_tq`, `real_1m_ivf_rabitq1_rerank`), 11 variants per
  scale, index shapes matching the Task 99 conventions with per-scale
  `nlists` (64/32 @10k, 64/128 @50k, 256/512 @1m — sqrt-ish
  convention, recorded in the generator).
- `t105-sweep-{10k,50k,1m}.json` — 71 steps per scale: the same
  45-cell matrix as the Task 99 profile minus the scale-independent
  QJL cells; iterations 300 (10k/50k) / 100 (1m); sweeps unchanged for
  cross-scale comparability. IVF "on" cells use the explicit flag.
  **Correction (per packet 006 aggregation + feedback):** the IVF
  "off" cells are **not** a kernel A/B column. The off arm omits
  `--ivf-scratch-soa-batch-decode`, and because the suite runner treats
  the `False` value the same as absent (only appends the flag when
  `True`), the Phase 1 default flip leaves batch decode ON in both
  arms. The IVF on/off pairs are therefore **same-config stability
  pairs** (they read 0 ± 4% at every scale), not a differential. The
  IVF kernel A/B evidence is Task 99's explicit pre-flip 100k run
  (`reviews/task-99/008|009`); a fresh non-100k IVF differential would
  need the `suite.rs` `False`→explicit-`off` fix plus a snapshot-restore
  rerun (out of scope here). The generator comment is annotated
  accordingly.
- `t105-g4-100k-rerun.json` — the AC3 confirmation column: packet
  004's neon-cap config with the cap GUC stripped (32 steps, incl.
  QJL), expecting the flipped default dispatch to reproduce the
  capped numbers.

Lane plans: G4 = 10k + 50k + 1m + the 100k confirmation rerun;
Intel = 10k + 50k + 1m (its 100k column from Task 99 stays valid —
x86 dispatch unchanged). Both lanes restore the post-Task-99
snapshots (100k + QJL fixtures already present).

Review asks: (1) the per-scale nlists convention; (2) 1m iteration
count (100) as the time/precision trade; (3) skipping QJL re-runs at
the new scales (dim-coverage fixture, scale-independent by design).
