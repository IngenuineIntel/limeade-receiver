-- ins_intro.sql

INSERT INTO metadata (
    ts_s
  , ts_ms
  , hostname
  , kernelver
  , distro
  , origin_user
  , processor
  , vendor
  , nr_cores
  , ram_mbs
  , swap_mbs
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
);
