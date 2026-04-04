CREATE TYPE tqvector;

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

CREATE TYPE tqvector (
    INTERNALLENGTH = variable,
    INPUT = tqvector_in,
    OUTPUT = tqvector_out,
    RECEIVE = tqvector_recv,
    SEND = tqvector_send,
    STORAGE = external
);

CREATE FUNCTION encode_to_tqvector(real[], integer, bigint)
RETURNS tqvector
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'encode_to_tqvector_wrapper';

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

CREATE FUNCTION tqhnsw_handler(internal)
RETURNS index_am_handler
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqhnsw_handler';

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

CREATE ACCESS METHOD tqhnsw TYPE INDEX HANDLER tqhnsw_handler;

CREATE OPERATOR CLASS tqvector_ip_ops
DEFAULT FOR TYPE tqvector USING tqhnsw AS
    OPERATOR 1 <#>(tqvector, real[]) FOR ORDER BY float_ops,
    FUNCTION 1 tqvector_query_inner_product(tqvector, real[]);
