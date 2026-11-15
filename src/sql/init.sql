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

CREATE TABLE proc_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT
  , snapshot_id INTEGER NOT NULL REFERENCES proc_snapshots(id)
  , attribute_id INTEGER NOT NULL REFERENCES proc_attributes(id)
  , pid INTEGER NOT NULL
  , threads INTEGER NOT NULL
  , ram_kb INTEGER NOT NULL
  , swap_kb INTEGER NOT NULL
  , state TEXT NOT NULL
  , cpu_user INTEGER NOT NULL
  , cpu_sys INTEGER NOT NULL
);

CREATE VIEW procs AS
SELECT samples.id, snapshots.ts_s, snapshots.ts_ms, samples.pid,
       attributes.ppid, attributes.ruid, attributes.euid, attributes.suid,
       attributes.fsuid, attributes.rgid, attributes.egid, attributes.sgid,
       attributes.fsgid, samples.threads, samples.ram_kb, samples.swap_kb,
       samples.state, samples.cpu_user, samples.cpu_sys, attributes.name,
       attributes.command
FROM proc_samples AS samples
JOIN proc_snapshots AS snapshots ON snapshots.id = samples.snapshot_id
JOIN proc_attributes AS attributes ON attributes.id = samples.attribute_id;

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

