create foreign data wrapper prometheus_wrapper
  handler prometheus_fdw_handler
  validator prometheus_fdw_validator;

create server test_server
  foreign data wrapper prometheus_wrapper
  options (
    address 'http://172.25.0.3:9090/',
    query 'scrape_duration_seconds'
  );

create foreign table hello (
  __name__ text,
   instance text,
    job text,
  value double precision
)
  server test_server;
