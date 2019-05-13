use std::time::Duration;

/// Ffi-safe equivalent of ::std::time::Duration .
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize, StableAbi,
)]
#[repr(C)]
pub struct RDuration {
    seconds: u64,
    subsec_nanos: u32,
}

impl RDuration {
    /// Constructs this RDuration from seconds and the nanoseconds inside a second .
    pub const fn new(seconds: u64, subsec_nanos: u32) -> Self {
        Self {
            subsec_nanos,
            seconds,
        }
    }

    /// The ammount of fractional nanoseconds (total_nanoseconds % 1_000_000_000) 
    /// of this RDuration.
    pub const fn subsec_nanos(&self) -> u32 {
        self.subsec_nanos
    }

    /// The ammount of seconds pf this RDuration.
    pub const fn seconds(&self) -> u64 {
        self.seconds
    }
}

impl_from_rust_repr! {
    impl From<Duration> for RDuration {
        fn(v){
            RDuration {
                subsec_nanos: v.subsec_nanos(),
                seconds: v.as_secs(),
            }
        }
    }
}

impl_into_rust_repr! {
    impl Into<Duration> for RDuration {
        fn(this){
            Duration::new(this.seconds, this.subsec_nanos)
        }
    }
}
