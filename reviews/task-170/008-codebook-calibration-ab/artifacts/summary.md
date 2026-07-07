# Task 170 Slice 3 codebook calibration A/B summary

Baseline install: `c24402b6ff7df7e8a6f79c5c0938d05315c51a6f` (post-renorm-revert, no codebook calibration).
After install: `a18c8c063d333e16ca3be3ab7ff15dfddb51b231` (TQ+ calibration plus coarse primary-posting fix).
Runner: `ecaz bench suite`, PG18 at `/Users/peter/.pgrx:28818`, scorer default `int8_approx`.

Conclusion: measured negative for promotion. Pure TQ recall improves, but latency is not neutral. stage2@25 recall is unchanged, with mixed latency at smaller scales and slight 100k latency regression. No 1m run was performed because the 100k gate did not show a latency-neutral win.

## TQ no-rerank pure default

### 10k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.9625 | 0.9594 | -0.31 pp |
| 16 | 0.9719 | 0.9688 | -0.31 pp |
| 24 | 0.9750 | 0.9750 | 0.00 pp |
| 32 | 0.9750 | 0.9750 | 0.00 pp |
| 48 | 0.9750 | 0.9750 | 0.00 pp |
| 64 | 0.9750 | 0.9750 | 0.00 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 0.88 | 0.88 | 0.00 | 0.0% |
| 40 | 1.05 | 1.09 | 0.04 | 3.8% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 168.0 MiB | 168.0 MiB | 0 | 0.0% |

### 50k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.9187 | 0.9250 | 0.63 pp |
| 16 | 0.9500 | 0.9594 | 0.94 pp |
| 24 | 0.9531 | 0.9656 | 1.25 pp |
| 32 | 0.9594 | 0.9688 | 0.94 pp |
| 48 | 0.9594 | 0.9688 | 0.94 pp |
| 64 | 0.9594 | 0.9688 | 0.94 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 1.32 | 1.79 | 0.47 | 35.6% |
| 40 | 1.36 | 2.08 | 0.72 | 52.9% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 836.4 MiB | 836.5 MiB | 104858 | 0.0% |

### 100k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.7844 | 0.8000 | 1.56 pp |
| 16 | 0.8344 | 0.8531 | 1.87 pp |
| 24 | 0.8750 | 0.8969 | 2.19 pp |
| 32 | 0.8938 | 0.9125 | 1.87 pp |
| 48 | 0.9125 | 0.9219 | 0.94 pp |
| 64 | 0.9250 | 0.9344 | 0.94 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 1.71 | 2.47 | 0.76 | 44.4% |
| 40 | 2.00 | 2.92 | 0.92 | 46.0% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 1.6 GiB | 1.6 GiB | 0 | 0.0% |

## stage2@25

### 10k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.9812 | 0.9812 | 0.00 pp |
| 16 | 0.9938 | 0.9938 | 0.00 pp |
| 24 | 1.0000 | 1.0000 | 0.00 pp |
| 32 | 1.0000 | 1.0000 | 0.00 pp |
| 48 | 1.0000 | 1.0000 | 0.00 pp |
| 64 | 1.0000 | 1.0000 | 0.00 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 0.71 | 0.88 | 0.17 | 23.9% |
| 40 | 0.76 | 0.84 | 0.08 | 10.5% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 170.5 MiB | 170.5 MiB | 0 | 0.0% |

### 50k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.9437 | 0.9437 | 0.00 pp |
| 16 | 0.9812 | 0.9812 | 0.00 pp |
| 24 | 0.9875 | 0.9875 | 0.00 pp |
| 32 | 0.9938 | 0.9938 | 0.00 pp |
| 48 | 0.9938 | 0.9938 | 0.00 pp |
| 64 | 0.9938 | 0.9938 | 0.00 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 1.24 | 1.17 | -0.07 | -5.6% |
| 40 | 1.40 | 1.34 | -0.06 | -4.3% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 848.0 MiB | 848.1 MiB | 104858 | 0.0% |

### 100k

Recall@10:
| nprobe | baseline | TQ+ | delta |
|---:|---:|---:|---:|
| 8 | 0.8156 | 0.8156 | 0.00 pp |
| 16 | 0.8750 | 0.8750 | 0.00 pp |
| 24 | 0.9187 | 0.9187 | 0.00 pp |
| 32 | 0.9375 | 0.9375 | 0.00 pp |
| 48 | 0.9563 | 0.9563 | 0.00 pp |
| 64 | 0.9719 | 0.9719 | 0.00 pp |

Latency mean:
| nprobe | baseline ms | TQ+ ms | delta ms | delta % |
|---:|---:|---:|---:|---:|
| 32 | 1.60 | 1.64 | 0.04 | 2.5% |
| 40 | 1.80 | 1.85 | 0.05 | 2.8% |

Storage total:
| baseline | TQ+ | delta bytes | delta % |
|---:|---:|---:|---:|
| 1.7 GiB | 1.7 GiB | 0 | 0.0% |

## Validation

- Tests: `cargo test --release --lib tqplus_coarse_rerank_dense_postings_keep_coarse_payload_width` passed.
- Tests: `cargo test --release --lib turboquant_calibrated_sidecar_scores_scalar_and_batch_consistently` passed before the coarse posting fix.
- Tests: `cargo test --release --lib coarse_rerank_accepts_tqplus_turboquant_sidecar_profile` passed before the coarse posting fix.
- Baseline suite rows: 168 in `artifacts/baseline/results.jsonl`.
- After suite rows: 168 in `artifacts/tqplus-fixed2/results.jsonl`.
- Baseline pre/post `ecaz_build_git_sha()`: `c24402b6ff7df7e8a6f79c5c0938d05315c51a6f`.
- After pre/post `ecaz_build_git_sha()`: `a18c8c063d333e16ca3be3ab7ff15dfddb51b231`.
