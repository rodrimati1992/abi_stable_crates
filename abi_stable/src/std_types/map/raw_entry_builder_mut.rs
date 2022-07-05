use super::*;

use std::{mem::ManuallyDrop, ptr};

use crate::{
    marker_type::UnsafeIgnoredType,
    prefix_type::{PrefixTypeTrait, WithMetadata},
    sabi_types::{RMut, RRef},
    std_types::Tuple2,
};

/// Note that the boxed type is just an alias here for consistency and ease of
/// use, but in reality it's just the unerased type, which in turn is just a
/// manually-dropped version of the raw entry.
pub type BoxedRRawEntryBuilderMut<'a, K, V, S> = UnerasedRawEntryBuilderMut<'a, K, V, S>;

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RRawEntryBuilderMut<'a, K, V, S> {
    raw_entry: RRef<'a, ErasedRawEntryBuilderMut<'a, K, V, S>>,
    vtable: RawEntryBuilderMutVTable<K, V, S>,
    _marker: UnsafeIgnoredType<RawEntryBuilderMut<'a, K, V, S>>,
}

// TODO: mutable version

/////////////////////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
struct ErasedRawEntryBuilderMut<'a, K, V, S>(PhantomData<(K, V)>, UnsafeIgnoredType<RRef<'a, S>>);

type UnerasedRawEntryBuilderMut<'a, K, V, S> = ManuallyDrop<RawEntryBuilderMut<'a, MapKey<K>, V, S>>;

impl<'a, K: 'a, V: 'a, S: 'a> ErasedType<'a> for ErasedRawEntryBuilderMut<'a, K, V, S> {
    type Unerased = UnerasedRawEntryBuilderMut<'a, K, V, S>;
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S> RRawEntryBuilderMut<'a, K, V, S> {
    fn vtable(&self) -> RawEntryBuilderMutVTable<K, V, S> {
        self.vtable
    }

    fn into_inner(self) -> RRef<'a, ErasedRawEntryBuilderMut<'a, K, V, S>> {
        let mut this = ManuallyDrop::new(self);
        unsafe { ((&this.raw_entry) as *const RRef<'a, ErasedRawEntryBuilderMut<'a, K, V, S>>).read() }
    }

    pub(super) unsafe fn new(raw_entry: &'a mut BoxedRRawEntryBuilderMut<'a, K, V, S>) -> Self {
        Self {
            raw_entry: ErasedRawEntryBuilderMut::from_unerased(raw_entry),
            vtable: RawEntryBuilderMutVTable::VTABLE_REF,
            _marker: UnsafeIgnoredType::DEFAULT,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

pub type MatchFn<K> = extern "C" fn(&K) -> bool;

impl<'a, K, V, S: BuildHasher> RRawEntryBuilderMut<'a, K, V, S> {
    /// TODO: docs
    pub fn from_key(&self, k: &K) -> ROption<Tuple2<&'a K, &'a V>>
    where
        S: BuildHasher,
    {
        todo!()
    }

    /// TODO: docs
    pub fn from_key_hashed(&self, hash: u64, k: &K) -> ROption<Tuple2<&'a K, &'a V>> {
        todo!()
    }

    /// TODO: docs
    pub fn from_hash(&self, hash: u64, is_match: MatchFn<K>) -> ROption<Tuple2<&'a K, &'a V>> {
        todo!()
    }
}

impl<K, V, S> Debug for RRawEntryBuilderMut<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RRawEntryBuilder").finish()
    }
}

impl<'a, K, V, S> Drop for RRawEntryBuilderMut<'a, K, V, S> {
    fn drop(&mut self) {
        let vtable = self.vtable();

        unsafe {
            vtable.drop_raw_entry()(self.raw_entry.reborrow());
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RawEntryBuilderMutVTable<K, V, S> {
    drop_raw_entry: for<'a> unsafe extern "C" fn(RMut<'a, ErasedRawEntryBuilderMut<'a, K, V, S>>),
    from_key: for<'a> extern "C" fn(RRawEntryBuilderMut<'a, K, V, S>, RRef<'a, K>) -> ROption<Tuple2<&'a K, &'a V>>,
    from_key_hashed_nocheck: for<'a> extern "C" fn(RRawEntryBuilderMut<'a, K, V, S>, u64, RRef<'a, K>) -> ROption<Tuple2<&'a K, &'a V>>,
    from_hash: for<'a> extern "C" fn(RRawEntryBuilderMut<'a, K, V, S>, u64, MatchFn<K>) -> ROption<Tuple2<&'a K, &'a V>>,
}

impl<K, V, S> RawEntryBuilderMutVTable<K, V, S> {
    const VTABLE_REF: RawEntryBuilderMutVTable_Ref<K, V, S> = RawEntryBuilderMutVTable_Ref(Self::WM_VTABLE.as_prefix());

    staticref! {
        const WM_VTABLE: WithMetadata<RawEntryBuilderMutVTable<K, V, S>> =
            WithMetadata::new(PrefixTypeTrait::METADATA, Self::VTABLE)
    }

    const VTABLE: RawEntryBuilderMutVTable<K, V, S> = RawEntryBuilderMutVTable {
        drop_raw_entry: ErasedRawEntryBuilderMut::drop_raw_entry,
        from_key: ErasedRawEntryBuilderMut::from_key,
        from_key_hashed_nocheck: ErasedRawEntryBuilderMut::from_key_hashed_nocheck,
        from_hash: ErasedRawEntryBuilderMut::from_hash,
    };
}

impl<'a, K, V, S> ErasedRawEntryBuilderMut<'a, K, V, S> {
    unsafe extern "C" fn drop_entry(this: RMut<'a, Self>) {
        extern_fn_panic_handling! {
            Self::run_downcast_as_mut(this, |this|{
                ManuallyDrop::drop(this);
            })
        }
    }
    extern "C" fn from_key<Q>(
        this: RRawEntryBuilderMut<'a, K, V, S>,
        k: RRef<'a, Q>,
    ) -> ROption<Tuple2<&'a K, &'a V>>
    where
        K: Borrow<Q>,
    {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as(
                    this.into_inner(),
                    |this| take_manuallydrop(this).from_key(k.into())
                )
            }
        }
    }
    extern "C" fn from_key_hashed_nocheck<Q>(
        this: RRawEntryBuilderMut<'a, K, V, S>,
        hash: u64,
        k: RRef<'a, Q>,
    ) -> ROption<Tuple2<&'a K, &'a V>>
    where
        K: Borrow<Q>,
    {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as(
                    this.into_inner(),
                    |this| take_manuallydrop(this).from_key_hashed_nocheck(hash, k.into())
                )
            }
        }
    }
    extern "C" fn from_hash<F>(
        this: RRawEntryBuilderMut<'a, K, V, S>,
        hash: u64,
        is_match: F,
    ) -> ROption<Tuple2<&'a K, &'a V>>
    where
        F: FnMut(&K) -> bool,
    {
        unsafe {
            extern_fn_panic_handling! {
                Self::run_downcast_as(
                    this.into_inner(),
                    |this| take_manuallydrop(this).from_hash(hash, is_match)
                )
            }
        }
    }
}

/// Copy paste of the unstable `ManuallyDrop::take`
unsafe fn take_manuallydrop<T>(slot: &ManuallyDrop<T>) -> T {
    ManuallyDrop::into_inner(ptr::read(slot))
}
