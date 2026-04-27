\pset pager off

CREATE OR REPLACE FUNCTION ec_ivf_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS '$libdir/ecaz', 'ec_ivf_handler';

CREATE ACCESS METHOD ec_ivf TYPE INDEX HANDLER ec_ivf_handler;

CREATE OPERATOR CLASS tqvector_ip_ops
DEFAULT FOR TYPE tqvector USING ec_ivf AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_ip_ops
DEFAULT FOR TYPE ecvector USING ec_ivf AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);

SELECT amname
FROM pg_am
WHERE amname IN ('ec_hnsw', 'ec_ivf')
ORDER BY amname;
