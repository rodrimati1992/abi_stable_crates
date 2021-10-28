//! Contains ffi-safe equivalent of `std::time::Duration`.

use std::time::Duration;

/// Ffi-safe equivalent of `std::time::Duration` .
///
/// # Example
///
/// ```
/// use abi_stable::std_types::RDuration;
///
/// let dur = RDuration::from_millis(31416);
/// assert_eq!(dur.as_secs(), 31);
/// assert_eq!(dur.as_nanos(), 31_416_000_000);
///
/// ```
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
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::new(1, 456_000_000);
    /// assert_eq!(dur.as_millis(), 1_456);
    /// assert_eq!(dur.as_micros(), 1_456_000);
    ///
    /// ```
    pub const fn new(seconds: u64, subsec_nanos: u32) -> Self {
        Self {
            subsec_nanos,
            seconds,
        }
    }

    /// Creates an RDuration of `secs` seconds.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_secs(14);
    /// assert_eq!(dur.as_millis(), 14_000);
    /// assert_eq!(dur.as_micros(), 14_000_000);
    ///
    /// ```
    pub const fn from_secs(secs: u64) -> RDuration {
        RDuration {
            seconds: secs,
            subsec_nanos: 0,
        }
    }

    /// Creates an RDuration of `milli` milliseconds.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_millis(628);
    /// assert_eq!(dur.as_micros(), 628_000);
    /// assert_eq!(dur.as_nanos(), 628_000_000);
    ///
    /// ```
    pub const fn from_millis(milli: u64) -> RDuration {
        RDuration {
            seconds: milli / 1000,
            subsec_nanos: (milli % 1000) as u32 * 1_000_000,
        }
    }

    /// Creates an RDuration of `micro` microseconds.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_micros(1024);
    /// assert_eq!(dur.as_millis(), 1);
    /// assert_eq!(dur.as_nanos(), 1024_000);
    ///
    /// ```
    pub const fn from_micros(micro: u64) -> RDuration {
        let million = 1_000_000;
        RDuration {
            seconds: micro / million,
            subsec_nanos: (micro % million) as u32 * 1000,
        }
    }

    /// Creates an RDuration of `nano` nanoseconds.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(128_256_512);
    /// assert_eq!(dur.as_millis(), 128);
    /// assert_eq!(dur.as_micros(), 128_256);
    ///
    /// ```
    pub const fn from_nanos(nano: u64) -> RDuration {
        let billion = 1_000_000_000;
        RDuration {
            seconds: nano / billion,
            subsec_nanos: (nano % billion) as u32,
        }
    }

    /// The amount of fractional nanoseconds (total_nanoseconds % 1_000_000_000)
    /// of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(64_128_256_512);
    /// assert_eq!(dur.subsec_nanos(), 128_256_512);
    ///
    /// ```
    pub const fn subsec_nanos(&self) -> u32 {
        self.subsec_nanos
    }

    /// The amount of seconds of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(64_128_256_512);
    /// assert_eq!(dur.seconds(), 64);
    ///
    /// ```
    pub const fn seconds(&self) -> u64 {
        self.seconds
    }

    /// The amount of seconds of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(64_128_256_512);
    /// assert_eq!(dur.as_secs(), 64);
    ///
    /// ```
    pub const fn as_secs(&self) -> u64 {
        self.seconds
    }

    /// The amount of milliseconds of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(64_128_256_512);
    /// assert_eq!(dur.as_millis(), 64_128);
    ///
    /// ```
    pub const fn as_millis(&self) -> u128 {
        self.seconds as u128 * 1_000_u128 + (self.subsec_nanos / 1_000_000) as u128
    }

    /// The amount of microseconds of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_nanos(64_128_256_512);
    /// assert_eq!(dur.as_micros(), 64_128_256);
    ///
    /// ```
    pub const fn as_micros(&self) -> u128 {
        self.seconds as u128 * 1_000_000_u128 + (self.subsec_nanos / 1000) as u128
    }

    /// The amount of nanoseconds of this RDuration.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RDuration;
    ///
    /// let dur = RDuration::from_micros(256);
    /// assert_eq!(dur.as_nanos(), 256_000);
    ///
    /// ```
    pub const fn as_nanos(&self) -> u128 {
        self.seconds as u128 * 1_000_000_000_u128 + self.subsec_nanos as u128
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
