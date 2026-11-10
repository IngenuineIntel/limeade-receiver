// compat.rs
// shims for liblimeade

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

    fn limeade_init_raw(ctx: &mut Context) -> Result<(), Error> {
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

    fn limeade_connect_raw(ctx: &mut Context) -> Result<(), Error> {
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
        ctx: &mut Context,
        recvd: &mut PacketData,
        wait_time: i32,
    ) -> Result<(), Error> {
        unsafe {
            match raw_lib::limeade_recv_wait(ctx, recvd, wait_time) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(Error::from_code(e)),
            }
        }
    }

    #[macro_export]
    macro_rules! pkt_type {
        ($pkt:expr) => {
            unsafe { $pkt.__bindgen_anon_1.packet_type }
        }
    }

    pub fn pkt_type(pkt: &PacketData) -> u8 {
      unsafe { pkt.__bindgen_anon_1.packet_type as u8 }
    }

    pub fn parse_intro(recvd: &mut PacketData) -> Result<Intro, Error> {
        unsafe {
            let mut intro: Intro = std::mem::zeroed();
            match raw_lib::limeade_parse_intro(&mut intro, recvd) as u32 {
                LIMEADE_OK!() => Ok(intro),
                e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn parse_events(recvd: &mut PacketData) -> Result<Events, Error> {
        unsafe {
            let mut events: Events = std::mem::zeroed();
            match raw_lib::limeade_parse_events(&mut events, recvd) as u32 {
              LIMEADE_OK!() => Ok(events),
              e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn parse_proc(recvd: &mut PacketData) -> Result<Procs, Error> {
        unsafe {
            let mut procs: Procs = std::mem::zeroed();
            match raw_lib::limeade_parse_proc_generic(&mut procs, recvd) as u32 {
              LIMEADE_OK!() => Ok(procs),
              e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn parse_perf(recvd: &mut PacketData) -> Result<Perf, Error> {
        unsafe {
            let mut perf: Perf = std::mem::zeroed();
            match raw_lib::limeade_parse_perf(&mut perf, recvd) as u32 {
              LIMEADE_OK!() => Ok(perf),
              e => Err(Error::from_code(e)),
            }
        }
    }

    pub fn release_recvd(recvd: &mut PacketData) {
        unsafe { raw_lib::limeade_release_recvd(recvd) }
    }

    pub fn destruct(ctx: &mut Context) {
        unsafe { raw_lib::limeade_destruct(ctx) }
    }
}
