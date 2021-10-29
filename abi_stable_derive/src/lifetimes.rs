abi_stable_shared::declare_tl_lifetime_types! {
    repr=usize,
    attrs=[
        derive(Hash),
    ]
}

/////////////////////////////////////////////////////////////////////

mod lifetime_counters;

pub(crate) use self::lifetime_counters::LifetimeCounters;

impl LifetimeRange {
    pub const DUMMY: Self = Self::from_range(Self::MAX_START..Self::MAX_START + Self::MAX_LEN);
}
