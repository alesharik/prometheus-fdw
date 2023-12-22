# Prometheus Foreign Data Wrapper
Postgres extension to fetch data from prometheus.

### Setup
```sql
create extension prometheusfdw; -- load extension

create foreign data wrapper prometheus_wrapper
  handler prometheus_fdw_handler
  validator prometheus_fdw_validator; -- create FDW

create server test_server
  foreign data wrapper prometheus_wrapper
  options (
    address 'http://172.25.0.3:9090/' -- server address
  ); -- connect to prometheus server

create foreign table scrape_duration_seconds (
    __name__ text, -- column name is fetched from labels
    instance text,
    job text,
    value double precision -- or from sample value
)
  server test_server
  options (
    query 'scrape_duration_seconds' -- use this query to fetch data
  ); -- create table
```

### Query format
Query can contain variables like `${var}`. These will be replaced by values from where clause.
If array of values is passed in `${var}`, it will be wrapped with `()` and concatenated with `|`.

### Where parameters
- `time timestamp` - when running instant query, specifies timestamp from where to take samples. Expects UTC.

### Table parameters
- `query` - PromQL query
- `rate` - (optional) rate parameter for VictoriaMetrics

### Todo
- [ ] Range queries