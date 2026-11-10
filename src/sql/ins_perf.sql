-- ins_perf.sql

INSERT INTO perf (
    perc_cpu_user
  , perc_cpu_sys
  , mem_total_kb
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
);
