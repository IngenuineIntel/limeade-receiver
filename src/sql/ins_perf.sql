-- ins_perf.sql

INSERT INTO perf (
    ts_s
  , ts_ms
  , perc_cpu_user
  , perc_cpu_sys
  , mem_free_kb
  , mem_available_kb
  , mem_cached_kb
) VALUES (
    ?1
  , ?2
  , ?3
  , ?4
  , ?5
  , ?6
  , ?7
);
