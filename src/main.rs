// main.rs

const VERSION: &'static str = "v0.1";
const LICENSE: &'static str = "AGPLv3";
const COPYRIGHT: &'static str = "Copyright (c) 2026 IngenuineIntel <Roan Rothrock>";

use std::fs::{File, OpenOptions};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::{thread, time::Duration};

mod compat;
mod config;
mod db;
mod log;

use compat::limeade;
use config::GlobalConfig;
use db::db::PacketSerializer;

fn input_closed(fd: libc::c_int) -> bool {
    let mut descriptor = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let ready = unsafe { libc::poll(&mut descriptor, 1, 0) };
    ready > 0
        && descriptor.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL) != 0
        && descriptor.revents & libc::POLLIN == 0
}

fn open_log_file(path: &str) -> std::io::Result<File> {
    match OpenOptions::new()
        .append(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
    {
        Ok(file) => {
            file.set_permissions(std::fs::Permissions::from_mode(0o660))?;
            Ok(file)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            OpenOptions::new().append(true).open(path)
        }
        Err(error) => Err(error),
    }
}

fn main() {
    /* Initializes configuration for the program */
    let config_file = File::open("/etc/limeade/limeade-receiver.ini").ok();
    let config = GlobalConfig::new(config_file);
    log::log_init(open_log_file(config.log_file()).ok());

    info!("+++ LIMEADE RECEIVER {VERSION} +++");
    info!("{COPYRIGHT}");
    warn!("Licensed under the {LICENSE} license");
    dbg!("Debugging is enabled");

    let store = match PacketSerializer::connect(config.db_directory()) {
        Ok(store) => store,
        Err(error) => {
            err!("Failed to start database: {error}");
            std::process::exit(1);
        }
    };

    let mut ctx = match limeade::init() {
        Ok(ctx) => ctx,
        Err(error) => {
            err!("{error}");
            std::process::exit(1);
        }
    };

    thread::sleep(Duration::from_millis(1000));

    loop {
        match limeade::recv_wait(&mut ctx, 2000) {
            Ok(mut recvd) => {
                match store.handle_packet(&mut recvd) {
                    Ok(()) => {}
                    Err(error) => {
                        err!("Failed to serialize packet: {error}");
                    }
                };
                limeade::release_recvd(&mut recvd);
            }
            Err(limeade::Error { code, message }) => {
                if code == limeade::ERROR_NO_DATA as u32 {
                    if input_closed(ctx.rfd) {
                        crit!("SSH client disconnected; shutting down");
                        break;
                    }
                } else {
                    err!("Failed to receive packet: {message}");
                    break;
                }
            }
        }
    }

    limeade::destruct(&mut ctx);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_closed_input_after_buffered_data_is_drained() {
        let mut descriptors = [0; 2];
        assert_eq!(unsafe { libc::pipe(descriptors.as_mut_ptr()) }, 0);
        assert_eq!(
            unsafe { libc::write(descriptors[1], b"x".as_ptr().cast(), 1) },
            1
        );
        unsafe { libc::close(descriptors[1]) };
        assert!(!input_closed(descriptors[0]));
        let mut byte = 0u8;
        assert_eq!(
            unsafe { libc::read(descriptors[0], (&mut byte as *mut u8).cast(), 1) },
            1
        );
        assert!(input_closed(descriptors[0]));
        unsafe { libc::close(descriptors[0]) };
    }

    #[test]
    fn log_file_is_group_writable_and_appends() {
        let directory = std::env::temp_dir().join(format!(
            "limeade-receiver-log-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let path = directory.join("receiver.log");
        let path = path.to_str().unwrap();
        let mut first = open_log_file(path).unwrap();
        assert_eq!(
            first.metadata().unwrap().permissions().mode() & 0o777,
            0o660
        );
        first.write_all(b"first").unwrap();
        open_log_file(path).unwrap().write_all(b" second").unwrap();
        first.write_all(b" third").unwrap();
        assert_eq!(std::fs::read_to_string(path).unwrap(), "first second third");
        assert!(open_log_file(&directory.join("missing/log").to_string_lossy()).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
