// compat.rs

mod raw_lib {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod limeade {
    use std::ffi::CStr;
    use std::fmt;

    use crate::compat::raw_lib;

    pub type Context    = raw_lib::limeade_context;
    pub type PacketData = raw_lib::limeade_packet_data;
    pub type Intro      = raw_lib::limeade_introduction;
    pub type Ack        = raw_lib::limeade_acknowledge;
    pub type Events     = raw_lib::limeade_events;
    pub type Event      = raw_lib::limeade_indiv_event;
    pub type Procs      = raw_lib::limeade_proc_generic;
    pub type Proc       = raw_lib::limeade_indiv_proc;
    pub type Perf       = raw_lib::limeade_perf;

    pub const TYPE_INTRO: u8  = raw_lib::limeade_packet_LIMEADE_PACKET_INTRODUCTION as u8;
    pub const TYPE_EVENTS: u8 = raw_lib::limeade_packet_LIMEADE_PACKET_EVENTS as u8;
    pub const TYPE_PROC: u8   = raw_lib::limeade_packet_LIMEADE_PACKET_PROC_GENERIC as u8;
    pub const TYPE_PERF: u8   = raw_lib::limeade_packet_LIMEADE_PACKET_PERF as u8;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Error {
        code: u32,
        message: String,
    }

    impl Error {
        fn from_code(code: u32) -> Self {
            let message = unsafe {
                CStr::from_ptr(raw_lib::LIMEADE_ERROR_REPRS[code as usize])
                    .to_string_lossy()
                    .into_owned()
            };
            Self { code, message }
        }

        pub fn code(&self) -> u32 {
            self.code
        }
    }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.message)
        }
    }

    impl std::error::Error for Error {}

    macro_rules! LIMEADE_OK {
        () => {
            raw_lib::limeade_error_LIMEADE_OK
        };
    }

    fn limeade_init_raw(ctx: &mut raw_lib::limeade_context) -> Result<(), Error> {
        unsafe {
            match raw_lib::limeade_init(
                ctx,
                (raw_lib::limeade_mode_LIMEADE_MODE_HOST_SSH
                    | raw_lib::limeade_compression_mode_LIMEADE_MODE_NO_COMPRESSION)
                    as i32,
            ) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(Error::from_code(e)),
            }
        }
    }

    fn limeade_connect_raw(ctx: &mut raw_lib::limeade_context) -> Result<(), Error> {
        unsafe {
            match raw_lib::limeade_connect(ctx) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn init() -> Result<Context, Error> {
        let mut ctx: Context = unsafe { std::mem::zeroed() };
        limeade_init_raw(&mut ctx)?;
        limeade_connect_raw(&mut ctx)?;
        Ok(ctx)
    }

    pub fn recv_wait(
        ctx: &mut raw_lib::limeade_context,
        recvd: &mut raw_lib::limeade_packet_data,
        wait_time: i32,
    ) -> Result<(), Error> {
        unsafe {
            match raw_lib::limeade_recv_wait(ctx, recvd, wait_time) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn release_recvd(recvd: &mut raw_lib::limeade_packet_data) {
        unsafe { raw_lib::limeade_release_recvd(recvd) }
    }

    pub fn destruct(ctx: &mut raw_lib::limeade_context) {
        unsafe { raw_lib::limeade_destruct(ctx) }
    }
}
