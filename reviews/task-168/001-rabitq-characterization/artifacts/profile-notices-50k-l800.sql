\pset pager off
\timing on

LOAD 'ecaz';
SET client_min_messages = notice;
SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_diskann.scan_profile_notice = on;
SET ec_diskann.list_size = 800;

DO $$
DECLARE
    q real[];
BEGIN
    FOR q IN
        SELECT source
        FROM t168_p1_real50k_diskann_queries
        ORDER BY id
        LIMIT 200
    LOOP
        PERFORM id
        FROM t168_p1_real50k_diskann_corpus
        ORDER BY embedding <#> q
        LIMIT 10;
    END LOOP;
END $$;

RESET ec_diskann.scan_profile_notice;
RESET ec_diskann.list_size;
RESET enable_sort;
RESET enable_bitmapscan;
RESET enable_seqscan;
