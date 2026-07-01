\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_repr_1m_idx'::regclass::oid, 2, 1, 'ecaz-spire-aws-39fef084-remote-1-20260621211856471900000006', decode('ae790a9a11d63382', 'hex'), 'ec_spire_aws_repr_1m_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_repr_1m_idx'::regclass::oid, 3, 1, 'ecaz-spire-aws-39fef084-remote-2-20260621211856471500000001', decode('00e4bf21d00e362a', 'hex'), 'ec_spire_aws_repr_1m_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

