CREATE TYPE tqvector;
CREATE TYPE ecvector;

CREATE FUNCTION tqvector_in(cstring)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_in_wrapper';

CREATE FUNCTION tqvector_out(tqvector)
RETURNS cstring
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_out_wrapper';

CREATE FUNCTION tqvector_recv(internal)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_recv_wrapper';

CREATE FUNCTION tqvector_send(tqvector)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_send_wrapper';

CREATE FUNCTION ecvector_in(cstring, oid, integer)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_in_wrapper';

CREATE FUNCTION ecvector_out(ecvector)
RETURNS cstring
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_out_wrapper';

CREATE FUNCTION ecvector_typmod_in(cstring[])
RETURNS integer
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_typmod_in';

CREATE FUNCTION ecvector_recv(internal, oid, integer)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_recv_wrapper';

CREATE FUNCTION ecvector_send(ecvector)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_send_wrapper';

CREATE TYPE tqvector (
    INTERNALLENGTH = variable,
    INPUT = tqvector_in,
    OUTPUT = tqvector_out,
    RECEIVE = tqvector_recv,
    SEND = tqvector_send,
    STORAGE = external
);

CREATE TYPE ecvector (
    INTERNALLENGTH = variable,
    INPUT = ecvector_in,
    OUTPUT = ecvector_out,
    TYPMOD_IN = ecvector_typmod_in,
    RECEIVE = ecvector_recv,
    SEND = ecvector_send,
    STORAGE = external
);

CREATE FUNCTION encode_to_tqvector(real[], integer, bigint)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'encode_to_tqvector_wrapper';

CREATE FUNCTION encode_to_ecvector(real[], integer, bigint)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'encode_to_ecvector_wrapper';

CREATE FUNCTION ecvector(ecvector, integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_coerce_wrapper';

CREATE FUNCTION ecvector_from_real_array(real[], integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_from_real_array_wrapper';

CREATE FUNCTION ecvector_to_real_array(ecvector, integer, boolean)
RETURNS real[]
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_to_real_array_wrapper';

CREATE FUNCTION ecvector_from_bytea(bytea, integer, boolean)
RETURNS ecvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_from_bytea_wrapper';

CREATE FUNCTION ecvector_to_bytea(ecvector, integer, boolean)
RETURNS bytea
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_to_bytea_wrapper';

CREATE CAST (ecvector AS ecvector)
WITH FUNCTION ecvector(ecvector, integer, boolean)
AS IMPLICIT;

CREATE CAST (real[] AS ecvector)
WITH FUNCTION ecvector_from_real_array(real[], integer, boolean)
AS ASSIGNMENT;

CREATE CAST (ecvector AS real[])
WITH FUNCTION ecvector_to_real_array(ecvector, integer, boolean);

CREATE CAST (bytea AS ecvector)
WITH FUNCTION ecvector_from_bytea(bytea, integer, boolean);

CREATE CAST (ecvector AS bytea)
WITH FUNCTION ecvector_to_bytea(ecvector, integer, boolean);

CREATE FUNCTION tqvector_inner_product(tqvector, tqvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_inner_product_wrapper';

CREATE FUNCTION tqvector_negative_inner_product(tqvector, tqvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_negative_inner_product_wrapper';

CREATE FUNCTION tqvector_query_inner_product(tqvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_query_inner_product_wrapper';

CREATE FUNCTION tqvector_negative_query_inner_product(tqvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_negative_query_inner_product_wrapper';

CREATE FUNCTION ecvector_inner_product(ecvector, ecvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_inner_product_wrapper';

CREATE FUNCTION ecvector_negative_inner_product(ecvector, ecvector)
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_negative_inner_product_wrapper';

CREATE FUNCTION ecvector_query_inner_product(ecvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_query_inner_product_wrapper';

CREATE FUNCTION ecvector_negative_query_inner_product(ecvector, real[])
RETURNS float4
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'ecvector_negative_query_inner_product_wrapper';

CREATE FUNCTION ec_hnsw_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_hnsw_handler';

CREATE FUNCTION ec_diskann_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'ec_diskann_handler';

CREATE OPERATOR <#> (
    PROCEDURE = tqvector_negative_inner_product,
    LEFTARG = tqvector,
    RIGHTARG = tqvector,
    COMMUTATOR = <#>
);

CREATE OPERATOR <#> (
    PROCEDURE = tqvector_negative_query_inner_product,
    LEFTARG = tqvector,
    RIGHTARG = real[]
);

CREATE OPERATOR <#> (
    PROCEDURE = ecvector_negative_inner_product,
    LEFTARG = ecvector,
    RIGHTARG = ecvector,
    COMMUTATOR = <#>
);

CREATE OPERATOR <#> (
    PROCEDURE = ecvector_negative_query_inner_product,
    LEFTARG = ecvector,
    RIGHTARG = real[]
);

CREATE ACCESS METHOD ec_hnsw TYPE INDEX HANDLER ec_hnsw_handler;
CREATE ACCESS METHOD ec_diskann TYPE INDEX HANDLER ec_diskann_handler;

CREATE OPERATOR CLASS tqvector_ip_ops
DEFAULT FOR TYPE tqvector USING ec_hnsw AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_ip_ops
DEFAULT FOR TYPE ecvector USING ec_hnsw AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);

CREATE OPERATOR CLASS tqvector_diskann_ip_ops
DEFAULT FOR TYPE tqvector USING ec_diskann AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);

CREATE OPERATOR CLASS ecvector_diskann_ip_ops
DEFAULT FOR TYPE ecvector USING ec_diskann AS
    OPERATOR 1 <#>(ecvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 ecvector_query_inner_product(ecvector, real[]);
