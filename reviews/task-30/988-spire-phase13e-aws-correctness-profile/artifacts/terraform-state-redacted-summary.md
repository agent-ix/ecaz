# Redacted Terraform State Summary

Source: local ignored file `terraform.tfstate.20260526T152209Z.archived`.

The full state file is intentionally not committed because it contains generated remote database passwords and secret strings.

Managed resource IDs recorded from the archived state:

```text
aws_iam_instance_profile.node	ecaz-spire-aws-node
aws_iam_role.node	ecaz-spire-aws-node
aws_iam_role_policy.node_artifacts_secrets	ecaz-spire-aws-node:ecaz-spire-aws-node-artifacts-secrets
aws_iam_role_policy_attachment.node_ssm	ecaz-spire-aws-node-20260525221948672200000008
aws_instance.coordinator	i-0916e18cc9fcab3b2
aws_instance.remote	i-032eeda9f40f9250e
aws_route_table.spire_aws_data	rtb-0cf7bceccf79a4d67
aws_route_table_association.spire_aws_data	rtbassoc-00957c376dc426470
aws_s3_bucket.artifacts	ecaz-spire-aws-20260525221947629900000007
aws_s3_bucket_server_side_encryption_configuration.artifacts	ecaz-spire-aws-20260525221947629900000007
aws_s3_bucket_versioning.artifacts	ecaz-spire-aws-20260525221947629900000007
aws_secretsmanager_secret.remote	arn:aws:secretsmanager:us-west-2:932658697181:secret:ecaz-spire-aws-90599215-remote-1-20260525221947626500000001-ZoRM5b
aws_secretsmanager_secret_version.remote	arn:aws:secretsmanager:us-west-2:932658697181:secret:ecaz-spire-aws-90599215-remote-1-20260525221947626500000001-ZoRM5b|terraform-20260525222014445000000015
aws_security_group.coordinator	sg-01b0acfd75c42b385
aws_security_group.endpoint	sg-02d4b46898beacab0
aws_security_group.remote	sg-01217649ffac4d6fe
aws_security_group_rule.coord_to_remote_pg	sgrule-3811928216
aws_subnet.spire_aws_data	subnet-03f972a52ab9872f4
aws_vpc.spire_aws	vpc-08e477285812abc44
aws_vpc_endpoint.ec2messages	vpce-02cc9b0402527d3be
aws_vpc_endpoint.s3	vpce-04e3bc7e4bbbf08e1
aws_vpc_endpoint.secretsmanager	vpce-060ddffd3c09517f3
aws_vpc_endpoint.ssm	vpce-0adb923948f003e4c
aws_vpc_endpoint.ssmmessages	vpce-045e16fd8d95e86e0
random_id.run	kFmSFQ
random_password.remote	none
```
