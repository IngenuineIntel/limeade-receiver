-- ins_intro.sql

INSERT INTO events (
    ts_s
  , ts_ms
  , pid
  , syscall
  , arg1
  , arg2
  , retval
) VALUES (
    ?1
  , ?2
  , ?3
  , ?4
  , ?5
  , ?6
  , ?7
);

