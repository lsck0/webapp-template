-- this user is used only internally for pgweb and grafana
DO
$do$
BEGIN
   IF NOT EXISTS (
      SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'root'
   ) THEN
      CREATE ROLE root WITH SUPERUSER LOGIN PASSWORD '1UdEQQTMBGCCWxvY2FsaG9zdIcEfwAAATAdBgNVHQ4EFgQUgGAHp2OojO06lrYu';
   END IF;
END
$do$;
