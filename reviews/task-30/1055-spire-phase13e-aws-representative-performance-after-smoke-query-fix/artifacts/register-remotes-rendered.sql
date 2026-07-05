\set ON_ERROR_STOP on

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_repr_1m_idx'::regclass::oid, 2, 1, 'ecaz-spire-aws-ad27322f-remote-1-20260528175902580200000005', decode('8d43b0fbfb7b69cd', 'hex'), 'ec_spire_aws_repr_1m_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_2;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_repr_1m_idx'::regclass::oid, 3, 1, 'ecaz-spire-aws-ad27322f-remote-2-20260528175902579900000003', decode('bf20f79bb101b4d7', 'hex'), 'ec_spire_aws_repr_1m_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_3;

SELECT ec_spire_register_remote_node_descriptor('ec_spire_aws_repr_1m_idx'::regclass::oid, 4, 1, 'ecaz-spire-aws-ad27322f-remote-3-20260528175902579700000001', decode('3d6ef37b9638a6cb', 'hex'), 'ec_spire_aws_repr_1m_remote_idx', 'active', 1, 1, '0.1.1', 'none') AS registered_node_4;

