-- ins_proc.sql

INSERT INTO procs (
    ts_s
  , ts_ms
  , pid
  , ppid
  , ruid
  , euid
  , suid
  , fsuid
  , rgid
  , egid
  , sgid
  , fsgid
  , threads
  , ram_kb
  , swap_kb
  , state
  , cpu_user
  , cpu_sys
  , name
  , command
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
  , ?10
  , ?11
  , ?12
  , ?13
  , ?14
  , ?15
  , ?16
  , ?17
  , ?18
  , ?19
  , ?20
);

