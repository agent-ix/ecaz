CREATE EXTENSION IF NOT EXISTS ecaz;
DROP TABLE IF EXISTS cs;
CREATE TABLE cs (id bigint, source real[], embedding ecvector);
INSERT INTO cs
SELECT g,
       arr,
       encode_to_ecvector(arr, 4, 42)
FROM (
  SELECT g, ARRAY[
      sin(g*0.11), cos(g*0.11), sin(g*0.23), cos(g*0.23),
      sin(g*0.37), cos(g*0.37), sin(g*0.71), cos(g*0.71)
  ]::real[] AS arr
  FROM generate_series(1, 400) g
) s;
CREATE INDEX cs_idx ON cs USING ec_distann (embedding ecvector_distann_ip_ops);
SELECT count(*) AS rows, (SELECT count(*) FROM pg_class WHERE relname='cs_idx') AS idx FROM cs;
