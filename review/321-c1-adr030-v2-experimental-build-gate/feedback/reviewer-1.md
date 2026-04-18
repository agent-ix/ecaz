## Feedback: ADR-030 v2 Experimental Build Gate

`TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` + `build_source_column` as double-gate for v2
builds.

### What's right

- Two gates, not one. An operator cannot accidentally build a v2 index by flipping
  just the env var — the source-column option also has to be set. That makes
  accidental v2 builds in production effectively impossible.
- Env var name is explicit about being experimental AND about which ADR it's gating.
  Searching for it in future incident postmortems will be easy.

### Concerns

1. **Gate is build-side only.** The scan gate (packet 323) is a format-version
   rejection, independent of env vars. That's correct: a built v2 index is a built v2
   index and has to be handled regardless of env. But it means turning off the env
   var after building a v2 index does not disable that index — it just prevents new
   v2 builds. Document this in the ADR so nobody assumes the env var is a "kill
   switch."

2. **Discoverability.** An operator setting the env var gets a v2 build with no user
   feedback that v2 is experimental beyond the name. Consider a server log line at
   build time ("ADR-030 v2 experimental grouped index format; not covered by upgrade
   guarantees") so there's an audit trail.

### Observation

Double-gating the experimental lane is the right conservative posture. Do not remove
the `build_source_column` gate until insert path and vacuum path are both grouped-v2
aware.
