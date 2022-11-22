use crate::{
    sabi_types::MaybeCmp,
    std_types::utypeid::{no_utypeid, some_utypeid, UTypeId},
};

/// Passed to trait object constructors to make the
/// trait object downcast capable,
/// as opposed to [`TD_Opaque`](./struct.TD_Opaque.html).
///
/// [The `from_value`/`from_ptr`/`from_const` methods here
/// ](../../docs/sabi_trait_inherent/index.html#methods) take this type.
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     sabi_trait::doc_examples::Action_TO,
///     std_types::RBox,
///     type_level::downcasting::TD_CanDowncast,
/// };
///
/// // The type annotation is purely for the reader.
/// let mut object: Action_TO<'static, RBox<()>> =
///     Action_TO::from_value(100_usize, TD_CanDowncast);
///
/// assert_eq!(object.obj.downcast_as::<u8>().ok(), None);
/// assert_eq!(object.obj.downcast_as::<char>().ok(), None);
/// assert_eq!(object.obj.downcast_as::<usize>().ok(), Some(&100_usize));
///
/// ```
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct TD_CanDowncast;

/// Passed to trait object constructors to make it impossible to downcast the
/// trait object,
/// as opposed to [`TD_CanDowncast`](./struct.TD_CanDowncast.html).
///
/// [The `from_value`/`from_ptr`/`from_const` methods here
/// ](../../docs/sabi_trait_inherent/index.html#methods) take this type.
///
/// # Example
///
/// ```rust
/// use abi_stable::{
///     sabi_trait::doc_examples::Action_TO,
///     std_types::RBox,
///     type_level::downcasting::TD_Opaque,
/// };
///
/// // The type annotation is purely for the reader.
/// let mut object: Action_TO<'static, RBox<()>> =
///     Action_TO::from_value(100_usize, TD_Opaque);
///
/// assert_eq!(object.obj.downcast_as::<u8>().ok(), None);
///
/// assert_eq!(object.obj.downcast_as::<char>().ok(), None);
///
/// // Because `Action_TO::from-value` was passed `TD_Opaque`,
/// // the trait object can't be downcasted
/// assert_eq!(object.obj.downcast_as::<usize>().ok(), None);
///
/// ```
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub struct TD_Opaque;

/// Gets a function optionally returning the `UTypeId` of `T`.
///
/// Whether the function returns `MaybeCmp::Just(typeid)` is determined by implementors:
///
/// - `TD_CanDowncast`: the function always returns `MaybeCmp::Just(typeid)`.
///
/// - `TD_Opaque`: the function always returns `MaybeCmp::Nothing`.
pub trait GetUTID<T> {
    /// the function.
    const UID: extern "C" fn() -> MaybeCmp<UTypeId>;
}

impl<T> GetUTID<T> for TD_CanDowncast
where
    T: 'static,
{
    const UID: extern "C" fn() -> MaybeCmp<UTypeId> = some_utypeid::<T>;
}

impl<T> GetUTID<T> for TD_Opaque {
    const UID: extern "C" fn() -> MaybeCmp<UTypeId> = no_utypeid;
}
