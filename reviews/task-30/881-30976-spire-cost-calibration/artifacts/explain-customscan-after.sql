SET enable_seqscan = off;
SET enable_bitmapscan = off;
SET enable_sort = off;
SET ec_spire.remote_tuple_transport = 'pg_binary_attr_v1';
SET ec_spire.nprobe = 8;

SELECT *
FROM ec_spire_custom_scan_index_eligibility('phase12_tuple_measure_coord_idx'::regclass);

EXPLAIN
SELECT id, title, body
FROM phase12_tuple_measure_corpus
ORDER BY embedding <#> ARRAY[0.10518387,0.113662176,0.01764,-0.09460031,-0.11986554,-0.034926936,0.082123324,0.12366978,0.05151481,-0.06800264,-0.12499878,-0.06707162,0.05252088,0.12382592,0.08128598,-0.035987914,-0.12017468,-0.093873404,0.01873465,0.11411816,0.10458195,-0.0011064137,-0.105777554,-0.1131973,-0.01654397,0.09531981,0.119546995,0.033863224,-0.082954235,-0.12350395,-0.050504707,0.06892834]::real[]
LIMIT 10;
