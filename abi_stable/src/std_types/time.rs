use std::time::Duration;

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Deserialize, Serialize, StableAbi,
)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
pub struct RDuration {
    seconds: u64,
    subsec_nanos: u32,
}

impl RDuration {
    pub fn new(seconds: u64, subsec_nanos: u32) -> Self {
        Self {
            subsec_nanos,
            seconds,
        }
    }

    pub fn subsec_nanos(&self) -> u32 {
        self.subsec_nanos
    }

    pub fn seconds(&self) -> u64 {
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
