CREATE OR REPLACE FUNCTION ec_diskann_index_graph_summary(index_oid oid)
RETURNS TABLE (metric text, value text)
STRICT STABLE
LANGUAGE c
AS '$libdir/ecaz', 'ec_diskann_index_graph_summary_wrapper';
