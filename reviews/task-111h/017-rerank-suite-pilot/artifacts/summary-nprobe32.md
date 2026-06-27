# Nprobe 32 Summary

| placement | format | width | recall@10 | p50 latency | index size | index B/row | total size |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| source | f32 | 32 | 0.9970 | 2.98 ms | 5.1 MiB | 538.2 | 164.2 MiB |
| source | f32 | 64 | 0.9985 | 3.64 ms | 5.1 MiB | 538.2 | 164.2 MiB |
| source | f32 | 128 | 0.9985 | 5.69 ms | 5.1 MiB | 538.2 | 164.2 MiB |
| source | f32 | 256 | 0.9985 | 9.27 ms | 5.1 MiB | 538.2 | 164.2 MiB |
| index | f16 | 32 | 0.9960 | 2.20 ms | 37.0 MiB | 3875.6 | 196.0 MiB |
| index | f16 | 64 | 0.9975 | 2.31 ms | 36.0 MiB | 3771.6 | 195.0 MiB |
| index | f16 | 128 | 0.9975 | 2.79 ms | 35.8 MiB | 3752.8 | 194.8 MiB |
| index | f16 | 256 | 0.9975 | 4.17 ms | 35.8 MiB | 3751.1 | 194.8 MiB |
| index | rabitq4 | 32 | 0.9775 | 1.73 ms | 14.7 MiB | 1544.2 | 173.8 MiB |
| index | rabitq4 | 64 | 0.9775 | 1.85 ms | 13.9 MiB | 1462.3 | 173.0 MiB |
| index | rabitq4 | 128 | 0.9775 | 2.18 ms | 13.8 MiB | 1445.9 | 172.8 MiB |
| index | rabitq4 | 256 | 0.9775 | 2.29 ms | 13.8 MiB | 1444.2 | 172.8 MiB |
| index | rabitq8 | 32 | 0.9845 | 2.75 ms | 22.3 MiB | 2334.7 | 181.3 MiB |
| index | rabitq8 | 64 | 0.9850 | 1.91 ms | 21.3 MiB | 2238.1 | 180.4 MiB |
| index | rabitq8 | 128 | 0.9850 | 2.75 ms | 21.2 MiB | 2219.2 | 180.2 MiB |
| index | rabitq8 | 256 | 0.9850 | 2.66 ms | 21.1 MiB | 2217.6 | 180.2 MiB |
| index | turboquant | 32 | 0.9730 | 2.09 ms | 14.7 MiB | 1540.1 | 173.7 MiB |
| index | turboquant | 64 | 0.9730 | 2.28 ms | 13.9 MiB | 1458.2 | 172.9 MiB |
| index | turboquant | 128 | 0.9730 | 2.43 ms | 13.7 MiB | 1439.3 | 172.8 MiB |
| index | turboquant | 256 | 0.9730 | 2.85 ms | 13.7 MiB | 1437.7 | 172.7 MiB |
