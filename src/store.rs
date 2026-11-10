use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

pub struct RecordStore {
    directory: PathBuf,
    file: Option<File>,
}

impl RecordStore {
    pub fn new(directory: impl Into<PathBuf>) -> io::Result<Self> {
        let directory = directory.into();
        fs::create_dir_all(&directory)?;
        Ok(Self {
            directory,
            file: None,
        })
    }

    pub fn set_client(&mut self, hostname: &str) -> io::Result<()> {
        let safe_name: String = hostname
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        let name = if safe_name.is_empty() || safe_name == "." || safe_name == ".." {
            "unknown"
        } else {
            &safe_name
        };
        self.file = Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o660)
                .open(self.directory.join(format!("{name}.jsonl")))?,
        );
        Ok(())
    }

    pub fn write_record(&mut self, record: &str) -> io::Result<()> {
        let file = match self.file.as_mut() {
            Some(file) => file,
            None => {
                self.file = Some(
                    OpenOptions::new()
                        .create(true)
                        .append(true)
                        .mode(0o660)
                        .open(self.directory.join("unknown.jsonl"))?,
                );
                self.file.as_mut().unwrap()
            }
        };
        writeln!(file, "{record}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_names_cannot_escape_store_directory() {
        let directory = std::env::temp_dir().join(format!("limeade-store-{}", std::process::id()));
        let mut store = RecordStore::new(&directory).unwrap();
        store.set_client("../../host name").unwrap();
        store.write_record("{}").unwrap();
        assert!(directory.join(".._.._host_name.jsonl").exists());
        fs::remove_dir_all(directory).unwrap();
    }
}