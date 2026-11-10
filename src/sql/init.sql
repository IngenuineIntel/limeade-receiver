-- init.sql

CREATE TABLE metadata (
    ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
  , hostname TEXT NOT NULL
  , kernelver TEXT NOT NULL
  , distro TEXT NOT NULL
  , origin_user TEXT NOT NULL
  , processor TEXT NOT NULL
  , vendor TEXT NOT NULL
  , nr_cores INTEGER NOT NULL
  , ram_mbs INTEGER NOT NULL
  , swap_mbs INTEGER NOT NULL
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT
  , ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
  , pid INTEGER NOT NULL
  , arg1 TEXT
  , arg2 TEXT
  , retval INTEGER NOT NULL
);

-- note: a single process in this table will be visible as a series of
-- different entries, as the distinction between processes will be done by any
-- program that parses this data
CREATE TABLE procs (
    id INTEGER PRIMARY KEY AUTOINCREMENT
  , ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
  , pid INTEGER NOT NULL
  , ppid INTEGER NOT NULL
  , ruid INTEGER NOT NULL
  , euid INTEGER NOT NULL
  , suid INTEGER NOT NULL
  , fsuid INTEGER NOT NULL
  , rgid INTEGER NOT NULL
  , egid INTEGER NOT NULL
  , sgid INTEGER NOT NULL
  , fsgid INTEGER NOT NULL
  , threads INTEGER NOT NULL
  , ram_kb INTEGER NOT NULL
  , swap_kb INTEGER NOT NULL
  , state TEXT NOT NULL
  , cpu_user INTEGER NOT NULL
  , cpu_sys INTEGER NOT NULL
  , name TEXT
  , command TEXT
);

CREATE TABLE perf (
    perc_cpu_user INTEGER NOT NULL
  , perc_cpu_sys INTEGER NOT NULL
  , mem_total_kb INTEGER NOT NULL
  , mem_free_kb INTEGER NOT NULL
  , mem_available_kb INTEGER NOT NULL
  , mem_cached_kb INTEGER NOT NULL
);

