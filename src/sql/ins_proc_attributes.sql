INSERT INTO proc_attributes (
    ppid, ruid, euid, suid, fsuid, rgid, egid, sgid, fsgid, name, command
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
ON CONFLICT (ppid, ruid, euid, suid, fsuid, rgid, egid, sgid, fsgid, name, command)
DO NOTHING;