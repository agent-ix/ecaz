# Artifact Manifest

Head SHA: `77ea192621b04417df2c13374c16f6db0a053a9c`

Packet: `review/30096-task28-ivf-current-auto-recommendation`

Timestamp: 2026-04-28 05:45 America/Los_Angeles

This is a synthesis packet. It did not run new SQL, recall, latency, or build
commands. Measurement claims are drawn from packet-local artifacts in the
source packets below.

## Source Packet 30084

Packet:

- `review/30084-task28-ivf-quantizer-headtohead-smoke`

Source artifacts:

- `artifacts/build_quantizer_surfaces.log`
- `artifacts/recall_turboquant.log`
- `artifacts/recall_pqfastscan.log`
- `artifacts/recall_rabitq.log`
- `artifacts/latency_turboquant.log`
- `artifacts/latency_pqfastscan.log`
- `artifacts/latency_rabitq_narrow.log`

Used for:

- Initial 10k TurboQuant / PQ-FastScan / RaBitQ smoke comparison.
- RaBitQ current latency caveat.

## Source Packet 30091

Packet:

- `review/30091-task28-ivf-100k-pqfastscan-turboquant-comparison`

Source artifacts:

- `artifacts/build_turboquant_100k_surface.log`
- `artifacts/recall_turboquant_100k_n64w25.log`
- `artifacts/latency_turboquant_100k_n64w25.log`
- Packet 30090 PQ-FastScan source artifacts cited by 30091.

Used for:

- 100k TurboQuant versus PQ-FastScan g8 n64 comparison.

## Source Packet 30094

Packet:

- `review/30094-task28-ivf-pqfastscan-g8-100k-n128-nprobe-middle`

Source artifacts:

- `artifacts/recall_g8_100k_n128_w500_p40_48_56_64.log`
- `artifacts/latency_g8_100k_n128_w500_p40_48_56_64.log`

Used for:

- Current n128 low-latency profile.
- Current n128 quality step-up profile.

## Source Packet 30095

Packet:

- `review/30095-task28-ivf-pqfastscan-g8-100k-nlists256`

Source artifacts:

- `artifacts/build_g8_100k_n256_surface.log`
- `artifacts/recall_g8_100k_n256_w500_p64_96_128.log`
- `artifacts/latency_g8_100k_n256_w500_p96_128.log`

Used for:

- Current n256 quality-biased profile and build-cost caveat.
