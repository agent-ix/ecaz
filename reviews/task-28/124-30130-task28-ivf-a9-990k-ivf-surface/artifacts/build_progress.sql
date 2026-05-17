\pset pager off
SELECT pid,
       state,
       wait_event_type,
       wait_event,
       now() - query_start AS query_age,
       left(query, 160) AS query
FROM pg_stat_activity
WHERE datname = 'postgres'
  AND query LIKE '%task28_ivf_pqg990k_g8_n128%'
ORDER BY query_start;

SELECT phase,
       lockers_total,
       lockers_done,
       blocks_total,
       blocks_done,
       tuples_total,
       tuples_done,
       partitions_total,
       partitions_done
FROM pg_stat_progress_create_index;
