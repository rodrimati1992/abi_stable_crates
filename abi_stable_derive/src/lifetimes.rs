abi_stable_shared::declare_tl_lifetime_types!{
    attrs=[
        derive(Hash),
    ]
}


/////////////////////////////////////////////////////////////////////


mod lifetime_set;
mod lifetime_counters;

pub(crate) use self::{
    lifetime_set::LifetimeSet,
    lifetime_counters::LifetimeCounters,
};