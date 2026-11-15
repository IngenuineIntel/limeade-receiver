-- ins_proc.sql

INSERT INTO proc_samples (
    snapshot_id
  , attribute_id
  , pid
  , threads
  , ram_kb
  , swap_kb
  , state
  , cpu_user
  , cpu_sys
) VALUES (
    ?1
  , ?2
  , ?3
  , ?4
  , ?5
  , ?6
  , ?7
  , ?8
  , ?9
);

