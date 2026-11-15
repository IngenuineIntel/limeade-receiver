SELECT id FROM proc_attributes
WHERE ppid = ?1 AND ruid = ?2 AND euid = ?3 AND suid = ?4 AND fsuid = ?5
  AND rgid = ?6 AND egid = ?7 AND sgid = ?8 AND fsgid = ?9
  AND name = ?10 AND command = ?11;