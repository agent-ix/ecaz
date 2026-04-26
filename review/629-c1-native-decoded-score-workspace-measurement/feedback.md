# Feedback: 629 Native Decoded Score Workspace Measurement

## Verdict: Accept

82% graph-phase reduction (303,530 ms → 54,199 ms serial) is a real win. The
mechanism is sound: predecoding 4-bit codes to f32 once per build removes
repeated nibble-unpack work from every candidate comparison in
`score_ip_codes_lite`. The result is consistent across serial and parallel paths,
confirming the hot path is the decode kernel, not overhead elsewhere.

## Activation Conditions

The 64 MiB workspace cap and the no-QJL 4-bit-only gate are the right safety
boundary. Source-scored builds (`ecvector` with raw source vectors) do not
enter this path — the fixture intentionally uses `tqvector` to isolate the
code-scored lane. The request explains this correctly.

## Parallel Build Note

Parallel and serial graph times are nearly identical (54,199 ms vs 54,361 ms).
This is expected: graph assembly is still leader-local serial work. The decoded
workspace reduces serial graph cost; it does not make graph assembly parallel.
The next real parallel gain requires concurrent DSM graph insertion, not further
per-node scoring optimization.

## No Issues
