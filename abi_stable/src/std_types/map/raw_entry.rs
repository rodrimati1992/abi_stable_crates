use super::*;

use std::{hash::Hash, mem::ManuallyDrop, ptr};

use crate::{
    marker_type::UnsafeIgnoredType,
    prefix_type::{PrefixTypeTrait, WithMetadata},
    sabi_types::{RMut, RRef},
    std_types::Tuple2,
};

/// Note that the boxed type is just an alias here for consistency and ease of
/// use, but in reality it's just the unerased type, which in turn is just a
/// manually-dropped version of the raw entry.
pub type BoxedRRawEntryBuilder<'a, K, V, S> = UnerasedRawEntryBuilder<'a, K, V, S>;

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound(K: 'a),
    bound(V: 'a),
    bound(S: 'a),
    // The hasher doesn't matter
    unsafe_unconstrained(S),
)]
pub struct RRawEntryBuilder<'a, K, V, S> {
    raw_entry: RRef<'a, ErasedRawEntryBuilder<'a, K, V, S>>,
    vtable: RawEntryVTable_Ref<K, V, S>,
    _marker: UnsafeIgnoredType<RawEntryBuilder<'a, K, V, S>>,
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
struct ErasedRawEntryBuilder<'a, K, V, S>(PhantomData<(K, V)>, UnsafeIgnoredType<RRef<'a, S>>);

type UnerasedRawEntryBuilder<'a, K, V, S> = ManuallyDrop<RawEntryBuilder<'a, MapKey<K>, V, S>>;

impl<'a, K: 'a, V: 'a, S: 'a> ErasedType<'a> for ErasedRawEntryBuilder<'a, K, V, S> {
    type Unerased = UnerasedRawEntryBuilder<'a, K, V, S>;
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S> RRawEntryBuilder<'a, K, V, S> {
    fn vtable(&self) -> RawEntryVTable_Ref<K, V, S> {
        self.vtable
    }

    fn into_inner(self) -> RRef<'a, ErasedRawEntryBuilder<'a, K, V, S>> {
        let mut this = ManuallyDrop::new(self);
        unsafe { ((&this.raw_entry) as *const RRef<'a, ErasedRawEntryBuilder<'a, K, V, S>>).read() }
    }

    pub(super) unsafe fn new(raw_entry: &'a BoxedRRawEntryBuilder<'a, K, V, S>) -> Self {
        Self {
            raw_entry: ErasedRawEntryBuilder::from_unerased(raw_entry),
            vtable: RawEntryVTable::VTABLE_REF,
            _marker: UnsafeIgnoredType::DEFAULT,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a, K, V, S: BuildHasher> RRawEntryBuilder<'a, K, V, S> {
    /// TODO: docs
    pub fn from_key<Q: ?Sized>(&self, k: &Q) -> ROption<Tuple2<&'a K, &'a V>>
    where
        S: BuildHasher,
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        RNone
    }

    /// TODO: docs
    pub fn from_key_hashed<Q: ?Sized>(&self, hash: u64, k: &Q) -> ROption<Tuple2<&'a K, &'a V>>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        RNone
    }

    /// TODO: docs
    pub fn from_hash<F>(&self, hash: u64, is_match: F) -> ROption<Tuple2<&'a K, &'a V>>
    where
        F: FnMut(&K) -> bool,
    {
        RNone
    }
}

impl<K, V, S> Debug for RRawEntryBuilder<'_, K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RRawEntryBuilder").finish()
    }
}

impl<'a, K, V, S> Drop for RRawEntryBuilder<'a, K, V, S> {
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
pub struct RawEntryVTable<K, V, S, F, Q>
    where
        K: Borrow<Q>,
        F: FnMut(&K) -> bool,
{
    drop_raw_entry: for<'a> unsafe extern "C" fn(RMut<'a, ErasedRawEntryBuilder<'a, K, V, S>>),
    from_key: for<'a> extern "C" fn(RRawEntryBuilder<'a, K, V, S>, RRef<'a, Q>) -> ROption<Tuple2<&'a K, &'a V>>,
    from_key_hashed_nocheck: for<'a> extern "C" fn(RRawEntryBuilder<'a, K, V, S>, u64, RRef<'a, Q>) -> ROption<Tuple2<&'a K, &'a V>>,
    from_hash: for<'a> extern "C" fn(RRawEntryBuilder<'a, K, V, S>, u64, F) -> ROption<Tuple2<&'a K, &'a V>>,
}

impl<K, V, S> RawEntryVTable<K, V, S> {
    const VTABLE_REF: RawEntryVTable_Ref<K, V, S> = RawEntryVTable_Ref(Self::WM_VTABLE.as_prefix());

    staticref! {
        const WM_VTABLE: WithMetadata<RawEntryVTable<K, V, S>> =
            WithMetadata::new(PrefixTypeTrait::METADATA, Self::VTABLE)
    }

    const VTABLE: RawEntryVTable<K, V, S> = RawEntryVTable {
        drop_raw_entry: ErasedRawEntryBuilder::drop_raw_entry,
        from_key: ErasedRawEntryBuilder::from_key,
        from_key_hashed_nocheck: ErasedRawEntryBuilder::from_key_hashed_nocheck,
        from_hash: ErasedRawEntryBuilder::from_hash,
    };
}

impl<'a, K, V, S> ErasedRawEntryBuilder<'a, K, V, S> {
    unsafe extern "C" fn drop_entry(this: RMut<'a, Self>) {
        extern_fn_panic_handling! {
            Self::run_downcast_as_mut(this, |this|{
                ManuallyDrop::drop(this);
            })
        }
    }
    extern "C" fn from_key<Q>(
        this: RRawEntryBuilder<'a, K, V, S>,
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
        this: RRawEntryBuilder<'a, K, V, S>,
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
        this: RRawEntryBuilder<'a, K, V, S>,
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
