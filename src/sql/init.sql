-- init.sql

PRAGMA foreign_keys = ON;

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
  , receiver_proc_pid INTEGER NOT NULL
);

CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT
  , ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
  , pid INTEGER NOT NULL
  , syscall TEXT NOT NULL
  , arg1 TEXT
  , arg2 TEXT
  , retval INTEGER NOT NULL
);

CREATE TABLE proc_snapshots (
    id INTEGER PRIMARY KEY
  , ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
);

CREATE TABLE proc_attributes (
    id INTEGER PRIMARY KEY
  , ppid INTEGER NOT NULL
  , ruid INTEGER NOT NULL
  , euid INTEGER NOT NULL
  , suid INTEGER NOT NULL
  , fsuid INTEGER NOT NULL
  , rgid INTEGER NOT NULL
  , egid INTEGER NOT NULL
  , sgid INTEGER NOT NULL
  , fsgid INTEGER NOT NULL
  , name TEXT NOT NULL
  , command TEXT NOT NULL
  , UNIQUE (ppid, ruid, euid, suid, fsuid, rgid, egid, sgid, fsgid, name, command)
);

CREATE TABLE proc_batches (
    snapshot_id INTEGER PRIMARY KEY REFERENCES proc_snapshots(id)
  , first_sample_id INTEGER NOT NULL CHECK (first_sample_id > 0)
  , sample_count INTEGER NOT NULL CHECK (sample_count BETWEEN 0 AND 65535)
  , data BLOB NOT NULL
);

CREATE TABLE perf (
    ts_s INTEGER NOT NULL
  , ts_ms INTEGER NOT NULL
  , perc_cpu_user INTEGER NOT NULL
  , perc_cpu_sys INTEGER NOT NULL
  , mem_total_kb INTEGER NOT NULL
  , mem_free_kb INTEGER NOT NULL
  , mem_available_kb INTEGER NOT NULL
  , mem_cached_kb INTEGER NOT NULL
);

