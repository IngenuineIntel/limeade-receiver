// compat.rs

mod raw_lib {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod limeade {
    use std::ffi::CStr;
    use std::fmt;

    use crate::compat::raw_lib;

    pub type LimeadeContext    = raw_lib::limeade_context;
    pub type LimeadePacketData = raw_lib::limeade_packet_data;
    pub type LimeadeIntro      = raw_lib::limeade_introduction;
    pub type LimeadeAck        = raw_lib::limeade_acknowledge;
    pub type LimeadeEvents     = raw_lib::limeade_events;
    pub type LimeadeEvent      = raw_lib::limeade_indiv_event;
    pub type LimeadeProcs      = raw_lib::limeade_proc_generic;
    pub type LimeadeProc       = raw_lib::limeade_indiv_proc;
    pub type LimeadePerf       = raw_lib::limeade_perf;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct LimeadeError {
        code: u32,
        message: String,
    }

    impl LimeadeError {
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

    impl fmt::Display for LimeadeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.message)
        }
    }

    impl std::error::Error for LimeadeError {}

    macro_rules! LIMEADE_OK {
        () => {
            raw_lib::limeade_error_LIMEADE_OK
        };
    }

    fn limeade_init_raw(ctx: &mut raw_lib::limeade_context) -> Result<(), LimeadeError> {
        unsafe {
            match raw_lib::limeade_init(
                ctx,
                (raw_lib::limeade_mode_LIMEADE_MODE_HOST_SSH
                    | raw_lib::limeade_compression_mode_LIMEADE_MODE_NO_COMPRESSION)
                    as i32,
            ) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(LimeadeError::from_code(e)),
            }
        }
    }

    fn limeade_connect_raw(ctx: &mut raw_lib::limeade_context) -> Result<(), LimeadeError> {
        unsafe {
            match raw_lib::limeade_connect(ctx) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(LimeadeError::from_code(e)),
            }
        }
    }

    pub fn init() -> Result<LimeadeContext, LimeadeError> {
        let mut ctx: LimeadeContext = unsafe { std::mem::zeroed() };
        limeade_init_raw(&mut ctx)?;
        limeade_connect_raw(&mut ctx)?;
        Ok(ctx)
    }

    pub fn recv_wait(
        ctx: &mut raw_lib::limeade_context,
        recvd: &mut raw_lib::limeade_packet_data,
        wait_time: i32,
    ) -> Result<(), LimeadeError> {
        unsafe {
            match raw_lib::limeade_recv_wait(ctx, recvd, wait_time) as u32 {
                LIMEADE_OK!() => Ok(()),
                e => Err(LimeadeError::from_code(e)),
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
