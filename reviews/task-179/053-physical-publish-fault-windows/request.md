# Review request: physical publish fault windows

## Scope

Please review implementation commit `e6e03dfc2` and the exact-commit canonical
suite evidence in this packet as the real three-instance FR-082/TC-042 publish
fault-window checkpoint.

The physical fixture previously proved Ready/Published topology and serving,
while its fault matrix still belonged to the historical replicated-control
branch. This checkpoint adds physical first-epoch publication faults on three
real PG18 owner processes:

- after the durable decision, crashes the last owner with `pg_ctl -m immediate`
  while publication proceeds in roster order;
- proves the local participant rolls back to `Ready`, the first remote owner
  durably commits `Published`, the last owner is unavailable, and the
  coordinator remains `Pending / Decided` with no active pointer;
- restarts the crashed owner, retries with
  `debug_fail_recover_after_publish_ack`, and proves both remote owners are
  `Published` while the local participant again rolls back to `Ready` and the
  coordinator still has no active pointer;
- disables injection and proves idempotent replay reaches `Applied / Published`
  with one active pointer and all three participants `Published`; and
- reruns exact topology, serving, and qualified remote frozen-row
  materialization after recovery.

The physical benchmark mode skips these scale-independent destructive drills,
so canonical 10k/50k/100k measurement runs remain unchanged. One-owner and
coordinator-outside-roster correctness cases also retain their existing direct
publish path.

The remote-owner proof now matches its documented purpose: it validates the
sampled UUID syntax, applies that identity qual with the sampled constant
vector, requires an `EcDistannDistributedScan`, and checks the materialized
remote UUID exactly. It no longer treats approximate global top-1 recall as a
lifecycle invariant.

## Result

The exact-commit release suite completes one step with zero failures, missing
artifacts, or stale steps. All four thresholds pass:

```text
participant down: Pending / Decided / active_count=0;
                  local=Ready, remote-acked=Published, node 3 unavailable
post all acks:    Pending / Decided / active_count=0;
                  owners=Ready,Published,Published
recovery:         Applied / Published / active_count=1;
                  owners=Published,Published,Published
topology:         33/24/33 records, zero non-owned rows/orphans,
                  both remote qualified materializations pass
```

This is the required commit-only behavior: partial participant acknowledgements
are not visible through the coordinator, and replay converges without rebuild
or source recapture.

## Validation

See `artifacts/manifest.md`. At exact SHA `e6e03dfc2`:

- `cargo check -p ecaz-cli` passes with the repository's existing unused-field
  warning;
- the focused suite parser test passes and normalizes all three new fault rows;
- the release CLI build passes and embeds the exact implementation SHA; and
- the canonical `ecaz bench suite` step plus final audit/status pass.

Warnings-denied whole-CLI clippy is not claimed: the current checkout has
unrelated pre-existing clippy failures in `ecaz-cloud`, build-probe, sidecar,
corpus, and other CLI modules. No clippy diagnostic named a new fault-drill
function; the exact compile, parser, and live suite are the scoped validation.

## Requested decision

Please confirm that this closes the remaining real three-instance
post-decision partial-ack and post-ack/pre-pointer crash windows for Task 179
acceptance criterion 6. This packet does not mark Task 179 complete: outside
review of the outstanding packets and reviewer-accepted Task 172 benchmark
evidence are still required.
