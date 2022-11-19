//! Contains an ffi-safe equivalent of `parking_lot::RwLock`.

use std::{
    cell::UnsafeCell,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

use lock_api::{RawRwLock as RawRwLockTrait, RawRwLockTimed};
use parking_lot::RawRwLock;

use super::{UnsafeOveralignedField, RAW_LOCK_SIZE};

use crate::{marker_type::UnsyncUnsend, prefix_type::WithMetadata, std_types::*, StableAbi};

///////////////////////////////////////////////////////////////////////////////

type OpaqueRwLock = UnsafeOveralignedField<RawRwLock, [u8; OM_PADDING]>;

const OM_PADDING: usize = RAW_LOCK_SIZE - mem::size_of::<RawRwLock>();

#[allow(clippy::declare_interior_mutable_const)]
const OPAQUE_LOCK: OpaqueRwLock =
    OpaqueRwLock::new(<RawRwLock as RawRwLockTrait>::INIT, [0u8; OM_PADDING]);

// assert rwlock size
const _: () = assert!(RAW_LOCK_SIZE == mem::size_of::<OpaqueRwLock>());

/// A read-write lock that allows dynamic mutable/shared borrows of shared data.
///
/// RRwLock allows either multiple shared locks,or a single write lock.
///
/// # Poisoning
///
/// As opposed to the standard library version of this type,
/// this rwlock type does not use poisoning,
/// simply unlocking the lock when a panic happens.
///
/// # Example
///
/// ```
/// use abi_stable::external_types::RRwLock;
///
/// static LOCK: RRwLock<usize> = RRwLock::new(0);
///
/// let guard = std::thread::spawn(|| {
///     for _ in 0..100 {
///         *LOCK.write() += 1;
///     }
/// });
///
/// for _ in 0..100 {
///     *LOCK.write() += 1;
/// }
///
/// guard.join().unwrap();
///
/// assert_eq!(*LOCK.read(), 200);
///
/// ```
///
#[repr(C)]
#[derive(StableAbi)]
pub struct RRwLock<T> {
    raw_lock: OpaqueRwLock,
    data: UnsafeCell<T>,
    vtable: VTable_Ref,
}

/// A read guard,which allows shared access to the data inside the `RRwLock`.
///
/// There can be many of these for the same RRwLock at any given time.
///
/// When dropped this will unlock the rwlock.
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
#[must_use]
pub struct RReadGuard<'a, T> {
    rlock: &'a RRwLock<T>,
    _marker: PhantomData<(&'a T, UnsyncUnsend)>,
}

/// A write guard,which allows mutable access to the data inside the `RRwLock`.
///
/// There can be only of these for the same RRwLock at any given time.
///
/// When dropped this will unlock the rwlock.
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
#[must_use]
pub struct RWriteGuard<'a, T> {
    rlock: &'a RRwLock<T>,
    _marker: PhantomData<(&'a mut T, UnsyncUnsend)>,
}

///////////////////////////////////////////////////////////////////////////////

impl<T> RRwLock<T> {
    /// Constructs a lock,wrapping `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// static LOCK: RRwLock<Option<String>> = RRwLock::new(None);
    ///
    /// let lock = RRwLock::new(0);
    ///
    /// ```
    pub const fn new(value: T) -> Self {
        Self {
            raw_lock: OPAQUE_LOCK,
            data: UnsafeCell::new(value),
            vtable: VTable::VTABLE,
        }
    }
}

#[allow(clippy::missing_const_for_fn)]
impl<T> RRwLock<T> {
    #[inline]
    fn vtable(&self) -> VTable_Ref {
        self.vtable
    }

    #[inline]
    fn write_guard(&self) -> RWriteGuard<'_, T> {
        RWriteGuard {
            rlock: self,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn read_guard(&self) -> RReadGuard<'_, T> {
        RReadGuard {
            rlock: self,
            _marker: PhantomData,
        }
    }

    /// Unwraps this lock into its wrapped data.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// let lock = RRwLock::new("hello".to_string());
    ///
    /// assert_eq!(lock.into_inner().as_str(), "hello");
    ///
    /// ```
    #[inline]
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Gets a mutable reference to its wrapped data.
    ///
    /// This does not require any locking,since it takes `self` mutably.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// let mut lock = RRwLock::new("Hello".to_string());
    ///
    /// lock.get_mut().push_str(", World!");
    ///
    /// assert_eq!(lock.read().as_str(), "Hello, World!");
    ///
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// Acquires a lock for reading,blocking the current thread until it can.
    ///
    /// This function returns a read guard,which releases read access when it is dropped.
    ///
    /// Trying to lock the rwlock for reading in the same thread that has write
    /// access to the same rwlock will cause a deadlock.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// static LOCK: RRwLock<usize> = RRwLock::new(0);
    ///
    /// *LOCK.write() += 4;
    ///
    /// let read_guard_a = LOCK.read();
    /// let read_guard_b = LOCK.read();
    ///
    /// assert_eq!(*read_guard_a, 4);
    /// assert_eq!(*read_guard_b, 4);
    ///
    /// ```
    ///
    #[inline]
    pub fn read(&self) -> RReadGuard<'_, T> {
        self.vtable().lock_shared()(&self.raw_lock);
        self.read_guard()
    }

    /// Attemps to acquire a lock for reading,failing if it is locked for writing.
    ///
    /// Returns the read guard if the rwlock can be immediately acquired,otherwise returns RNone.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// static LOCK: RRwLock<usize> = RRwLock::new(0);
    ///
    /// let mut write_guard = LOCK.write();
    ///
    /// assert!(LOCK.try_read().is_none());
    ///
    /// *write_guard += 4;
    /// drop(write_guard);
    ///
    /// assert_eq!(*LOCK.try_read().unwrap(), 4);
    ///
    /// ```
    ///
    #[inline]
    pub fn try_read(&self) -> ROption<RReadGuard<'_, T>> {
        if self.vtable().try_lock_shared()(&self.raw_lock) {
            RSome(self.read_guard())
        } else {
            RNone
        }
    }

    /// Attempts to acquire a lock for reading,for the timeout duration.
    ///
    /// Once the timeout is reached,this will return None,
    /// otherwise it will return the read guard.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RRwLock, std_types::RDuration};
    ///
    /// static LOCK: RRwLock<usize> = RRwLock::new(0);
    ///
    /// static DUR: RDuration = RDuration::from_millis(1);
    ///
    /// let mut write_guard = LOCK.write();
    ///
    /// assert!(LOCK.try_read_for(DUR).is_none());
    ///
    /// *write_guard += 7;
    /// drop(write_guard);
    ///
    /// assert_eq!(*LOCK.try_read_for(DUR).unwrap(), 7);
    ///
    /// ```
    #[inline]
    pub fn try_read_for(&self, timeout: RDuration) -> ROption<RReadGuard<'_, T>> {
        if self.vtable().try_lock_shared_for()(&self.raw_lock, timeout) {
            RSome(self.read_guard())
        } else {
            RNone
        }
    }

    /// Acquires a lock for writing,blocking the current thread until it can.
    ///
    /// This function returns a write guard,which releases write access when it is dropped.
    ///
    /// Trying to lock the rwlock in the same thread that has read or write
    /// access to the same rwlock will cause a deadlock.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// let lock = RRwLock::new(0);
    ///
    /// let mut guard = lock.write();
    ///
    /// *guard += 4;
    ///
    /// assert_eq!(*guard, 4);
    ///
    /// ```
    #[inline]
    pub fn write(&self) -> RWriteGuard<'_, T> {
        self.vtable().lock_exclusive()(&self.raw_lock);
        self.write_guard()
    }

    /// Attemps to acquire a lock for writing.
    ///
    /// Returns the write guard if the rwlock can be immediately acquired,otherwise returns RNone.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RRwLock;
    ///
    /// let lock = RRwLock::new(0);
    ///
    /// let mut guard = lock.write();
    ///
    /// assert!(lock.try_write().is_none());
    ///
    /// *guard += 4;
    ///
    /// assert_eq!(*guard, 4);
    ///
    /// ```
    #[inline]
    pub fn try_write(&self) -> ROption<RWriteGuard<'_, T>> {
        if self.vtable().try_lock_exclusive()(&self.raw_lock) {
            RSome(self.write_guard())
        } else {
            RNone
        }
    }

    /// Attempts to acquire a lock for writing,for the timeout duration.
    ///
    /// Once the timeout is reached,this will return None,
    /// otherwise it will return the write guard.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RRwLock, std_types::RDuration};
    ///
    /// static DUR: RDuration = RDuration::from_millis(1);
    ///
    /// let lock = RRwLock::new(0);
    ///
    /// let mut write_guard = lock.try_write_for(DUR).unwrap();
    /// *write_guard += 4;
    ///
    /// assert!(lock.try_write_for(DUR).is_none());
    ///
    /// assert_eq!(*write_guard, 4);
    ///
    /// ```
    ///
    #[inline]
    pub fn try_write_for(&self, timeout: RDuration) -> ROption<RWriteGuard<'_, T>> {
        if self.vtable().try_lock_exclusive_for()(&self.raw_lock, timeout) {
            RSome(self.write_guard())
        } else {
            RNone
        }
    }
}

unsafe impl<T: Send> Send for RRwLock<T> where RawRwLock: Send {}

unsafe impl<T: Send + Sync> Sync for RRwLock<T> where RawRwLock: Sync {}

///////////////////////////////////////////////////////////////////////////////

impl<T: Default> Default for RRwLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

///////////////////////////////////////////////////////////////////////////////

macro_rules! impl_lock_guard {
    ($guard:ident) => {
        impl<'a, T> Display for $guard<'a, T>
        where
            T: Display,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Display::fmt(&**self, f)
            }
        }

        impl<'a, T> Debug for $guard<'a, T>
        where
            T: Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                Debug::fmt(&**self, f)
            }
        }

        impl<'a, T> Deref for $guard<'a, T> {
            type Target = T;

            fn deref(&self) -> &T {
                unsafe { &*self.rlock.data.get() }
            }
        }
    };
}

//////////////////////////////////////

impl_lock_guard! { RReadGuard  }

impl<'a, T> Drop for RReadGuard<'a, T> {
    fn drop(&mut self) {
        let vtable = self.rlock.vtable();
        vtable.unlock_shared()(&self.rlock.raw_lock);
    }
}

//////////////////////////////////////

impl_lock_guard! { RWriteGuard }
impl<'a, T> DerefMut for RWriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rlock.data.get() }
    }
}

impl<'a, T> Drop for RWriteGuard<'a, T> {
    fn drop(&mut self) {
        let vtable = self.rlock.vtable();
        vtable.unlock_exclusive()(&self.rlock.raw_lock);
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
struct VTable {
    lock_shared: extern "C" fn(this: &OpaqueRwLock),
    try_lock_shared: extern "C" fn(this: &OpaqueRwLock) -> bool,
    try_lock_shared_for: extern "C" fn(this: &OpaqueRwLock, timeout: RDuration) -> bool,
    unlock_shared: extern "C" fn(this: &OpaqueRwLock),

    lock_exclusive: extern "C" fn(this: &OpaqueRwLock),
    try_lock_exclusive: extern "C" fn(this: &OpaqueRwLock) -> bool,
    try_lock_exclusive_for: extern "C" fn(this: &OpaqueRwLock, timeout: RDuration) -> bool,
    #[sabi(last_prefix_field)]
    unlock_exclusive: extern "C" fn(this: &OpaqueRwLock),
}

impl VTable {
    const _TMP0: WithMetadata<VTable> = {
        let vtable = VTable {
            lock_shared,
            try_lock_shared,
            try_lock_shared_for,
            unlock_shared,
            lock_exclusive,
            try_lock_exclusive,
            try_lock_exclusive_for,
            unlock_exclusive,
        };
        WithMetadata::new(vtable)
    };

    // The VTABLE for this type in this executable/library
    const VTABLE: VTable_Ref = { VTable_Ref(Self::_TMP0.static_as_prefix()) };
}

extern "C" fn lock_shared(this: &OpaqueRwLock) {
    extern_fn_panic_handling! {
        this.value.lock_shared();
    }
}
extern "C" fn try_lock_shared(this: &OpaqueRwLock) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock_shared()
    }
}
extern "C" fn try_lock_shared_for(this: &OpaqueRwLock, timeout: RDuration) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock_shared_for(timeout.into())
    }
}
extern "C" fn unlock_shared(this: &OpaqueRwLock) {
    extern_fn_panic_handling! {
        unsafe{
            this.value.unlock_shared();
        }
    }
}

extern "C" fn lock_exclusive(this: &OpaqueRwLock) {
    extern_fn_panic_handling! {
        this.value.lock_exclusive();
    }
}
extern "C" fn try_lock_exclusive(this: &OpaqueRwLock) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock_exclusive()
    }
}
extern "C" fn try_lock_exclusive_for(this: &OpaqueRwLock, timeout: RDuration) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock_exclusive_for(timeout.into())
    }
}
extern "C" fn unlock_exclusive(this: &OpaqueRwLock) {
    extern_fn_panic_handling! {
        unsafe{
            this.value.unlock_exclusive();
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests {
    use super::*;

    use std::{thread, time::Duration};

    use crossbeam_utils::thread::scope as scoped_thread;

    use crate::test_utils::check_formatting_equivalence;

    #[test]
    fn get_mut() {
        let mut lock: RRwLock<usize> = RRwLock::new(0);
        assert_eq!(*lock.read(), 0);
        *lock.get_mut() += 100;
        assert_eq!(*lock.read(), 100);
        *lock.get_mut() += 100;
        assert_eq!(*lock.read(), 200);
    }

    #[test]
    fn into_inner() {
        let lock: RRwLock<usize> = RRwLock::new(0);
        assert_eq!(*lock.read(), 0);
        *lock.write() += 100;
        assert_eq!(*lock.read(), 100);
        assert_eq!(lock.into_inner(), 100);
    }

    #[test]
    fn debug_display() {
        let str_ = "\nhello\rhello\rhello\n";
        let lock = RRwLock::new(str_);
        check_formatting_equivalence(&lock.read(), str_);
        check_formatting_equivalence(&lock.write(), str_);
    }

    const EXPECTED: usize = 64;

    #[test]
    #[cfg(not(all(miri, target_os = "windows")))]
    fn regular_locking() {
        static LOCK: RRwLock<usize> = RRwLock::new(0);

        scoped_thread(|scope| {
            for j in 0..16 {
                scope.spawn(move |_| {
                    for _ in 0..8 {
                        if (j % 2) == 0 {
                            *LOCK.write() += 1;
                        } else {
                            let value = *LOCK.read();
                            assert!(value <= EXPECTED, "{} <= {}", value, EXPECTED);
                        }
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(*LOCK.read(), 64);
    }

    #[test]
    #[cfg(not(all(miri, target_os = "windows")))]
    fn try_lock__() {
        static LOCK: RRwLock<usize> = RRwLock::new(0);

        scoped_thread(|scope| {
            for j in 0..16 {
                scope.spawn(move |_| {
                    for _ in 0..8 {
                        loop {
                            if (j % 2) == 0 {
                                if let RSome(mut guard) = LOCK.try_write() {
                                    *guard += 1;
                                    break;
                                }
                            } else if let RSome(guard) = LOCK.try_read() {
                                assert!(*guard <= EXPECTED, "{} <= {}", *guard, EXPECTED);
                                break;
                            }
                        }
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(*LOCK.read(), 64);

        scoped_thread(|scope| {
            let _guard = LOCK.write();
            scope.spawn(move |_| {
                assert_eq!(LOCK.try_read().map(drop), RNone);
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();

        scoped_thread(|scope| {
            let _guard = LOCK.read();
            scope.spawn(move |_| {
                assert_eq!(LOCK.try_write().map(drop), RNone);
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();
    }

    // miri detects a memory leak here, and I can't figure out why
    #[test]
    #[cfg(not(miri))]
    fn try_lock_for() {
        static LOCK: RRwLock<usize> = RRwLock::new(0);

        scoped_thread(|scope| {
            for j in 0..16 {
                scope.spawn(move |_| {
                    for i in 0..8 {
                        let wait_for = RDuration::new(0, (i + 1) * 500_000);
                        loop {
                            if (j % 2) == 0 {
                                if let RSome(mut guard) = LOCK.try_write_for(wait_for) {
                                    *guard += 1;
                                    break;
                                }
                            } else if let RSome(guard) = LOCK.try_read_for(wait_for) {
                                assert!(*guard <= EXPECTED, "{} <= {}", *guard, EXPECTED);
                                break;
                            }
                        }
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(*LOCK.read(), 64);

        scoped_thread(|scope| {
            let _guard = LOCK.write();
            scope.spawn(move |_| {
                assert_eq!(
                    LOCK.try_read_for(RDuration::new(0, 100_000)).map(drop),
                    RNone
                );
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();

        scoped_thread(|scope| {
            let _guard = LOCK.read();
            scope.spawn(move |_| {
                assert_eq!(
                    LOCK.try_write_for(RDuration::new(0, 100_000)).map(drop),
                    RNone
                );
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();
    }
}
