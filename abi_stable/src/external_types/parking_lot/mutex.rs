//! Contains an ffi-safe equivalent of `parking_lot::Mutex`.

use std::{
    cell::UnsafeCell,
    fmt::{self, Debug, Display},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
};

use lock_api::{RawMutex as RawMutexTrait, RawMutexTimed};
use parking_lot::RawMutex;

use super::{UnsafeOveralignedField, RAW_LOCK_SIZE};

use crate::{marker_type::UnsyncUnsend, prefix_type::WithMetadata, std_types::*, StableAbi};

///////////////////////////////////////////////////////////////////////////////

type OpaqueMutex = UnsafeOveralignedField<RawMutex, [u8; OM_PADDING]>;

const OM_PADDING: usize = RAW_LOCK_SIZE - mem::size_of::<RawMutex>();

#[allow(clippy::declare_interior_mutable_const)]
const OPAQUE_MUTEX: OpaqueMutex =
    OpaqueMutex::new(<RawMutex as RawMutexTrait>::INIT, [0u8; OM_PADDING]);

// assert_mutex_size
const _: () = assert!(RAW_LOCK_SIZE == mem::size_of::<OpaqueMutex>());

/// A mutual exclusion lock that allows dynamic mutable borrows of shared data.
///
/// # Poisoning
///
/// As opposed to the standard library version of this type,
/// this mutex type does not use poisoning,
/// simply unlocking the lock when a panic happens.
///
/// # Example
///
/// ```
/// use abi_stable::external_types::RMutex;
///
/// static MUTEX: RMutex<usize> = RMutex::new(0);
///
/// let guard = std::thread::spawn(|| {
///     for _ in 0..100 {
///         *MUTEX.lock() += 1;
///     }
/// });
///
/// for _ in 0..100 {
///     *MUTEX.lock() += 1;
/// }
///
/// guard.join().unwrap();
///
/// assert_eq!(*MUTEX.lock(), 200);
///
/// ```
#[repr(C)]
#[derive(StableAbi)]
pub struct RMutex<T> {
    raw_mutex: OpaqueMutex,
    data: UnsafeCell<T>,
    vtable: VTable_Ref,
}

/// A mutex guard,which allows mutable access to the data inside an `RMutex`.
///
/// When dropped this will unlock the mutex.
///
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
#[must_use]
pub struct RMutexGuard<'a, T> {
    rmutex: &'a RMutex<T>,
    _marker: PhantomData<(&'a mut T, UnsyncUnsend)>,
}

///////////////////////////////////////////////////////////////////////////////

impl<T> RMutex<T> {
    /// Constructs a mutex,wrapping `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// static MUTEX: RMutex<Option<String>> = RMutex::new(None);
    ///
    /// let mutex = RMutex::new(0);
    ///
    /// ```
    pub const fn new(value: T) -> Self {
        Self {
            raw_mutex: OPAQUE_MUTEX,
            data: UnsafeCell::new(value),
            vtable: VTable::VTABLE,
        }
    }

    #[inline]
    const fn vtable(&self) -> VTable_Ref {
        self.vtable
    }

    #[inline]
    fn make_guard(&self) -> RMutexGuard<'_, T> {
        RMutexGuard {
            rmutex: self,
            _marker: PhantomData,
        }
    }

    /// Unwraps this mutex into its wrapped data.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// let mutex = RMutex::new("hello".to_string());
    ///
    /// assert_eq!(mutex.into_inner().as_str(), "hello");
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
    /// use abi_stable::external_types::RMutex;
    ///
    /// let mut mutex = RMutex::new("Hello".to_string());
    ///
    /// mutex.get_mut().push_str(", World!");
    ///
    /// assert_eq!(mutex.lock().as_str(), "Hello, World!");
    ///
    /// ```
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }

    /// Acquires a mutex,blocking the current thread until it can.
    ///
    /// This function returns a guard which releases the mutex when it is dropped.
    ///
    /// Trying to lock the mutex in the same thread that holds the lock will cause a deadlock.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// static MUTEX: RMutex<usize> = RMutex::new(0);
    ///
    /// let guard = std::thread::spawn(|| *MUTEX.lock() += 1);
    ///
    /// *MUTEX.lock() += 4;
    ///
    /// guard.join().unwrap();
    ///
    /// assert_eq!(*MUTEX.lock(), 5);
    ///
    /// ```
    #[inline]
    pub fn lock(&self) -> RMutexGuard<'_, T> {
        self.vtable().lock()(&self.raw_mutex);
        self.make_guard()
    }

    /// Attemps to acquire a mutex guard.
    ///
    /// Returns the mutex guard if the mutex can be immediately acquired,
    /// otherwise returns `RNone`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// static MUTEX: RMutex<usize> = RMutex::new(0);
    ///
    /// let mut guard = MUTEX.try_lock().unwrap();
    ///
    /// assert!(MUTEX.try_lock().is_none());
    ///
    /// assert_eq!(*guard, 0);
    ///
    /// ```
    ///
    #[inline]
    pub fn try_lock(&self) -> ROption<RMutexGuard<'_, T>> {
        if self.vtable().try_lock()(&self.raw_mutex) {
            RSome(self.make_guard())
        } else {
            RNone
        }
    }

    /// Attempts to acquire a mutex guard for the `timeout` duration.
    ///
    /// Once the timeout is reached,this will return `RNone`,
    /// otherwise it will return the mutex guard.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{external_types::RMutex, std_types::RDuration};
    ///
    /// static MUTEX: RMutex<usize> = RMutex::new(0);
    ///
    /// static DUR: RDuration = RDuration::from_millis(4);
    ///
    /// let mut guard = MUTEX.try_lock_for(DUR).unwrap();
    ///
    /// assert!(MUTEX.try_lock_for(DUR).is_none());
    ///
    /// assert_eq!(*guard, 0);
    ///
    /// ```
    ///
    #[inline]
    pub fn try_lock_for(&self, timeout: RDuration) -> ROption<RMutexGuard<'_, T>> {
        if self.vtable().try_lock_for()(&self.raw_mutex, timeout) {
            RSome(self.make_guard())
        } else {
            RNone
        }
    }
}

unsafe impl<T: Send> Send for RMutex<T> where RawMutex: Send {}

unsafe impl<T: Send> Sync for RMutex<T> where RawMutex: Sync {}

///////////////////////////////////////////////////////////////////////////////

impl<T: Default> Default for RMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

///////////////////////////////////////////////////////////////////////////////

impl<'a, T> Display for RMutexGuard<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<'a, T> Debug for RMutexGuard<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<'a, T> Deref for RMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.rmutex.data.get() }
    }
}

impl<'a, T> DerefMut for RMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.rmutex.data.get() }
    }
}

impl<'a, T> Drop for RMutexGuard<'a, T> {
    fn drop(&mut self) {
        let vtable = self.rmutex.vtable();
        vtable.unlock()(&self.rmutex.raw_mutex);
    }
}

///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
#[sabi(missing_field(panic))]
struct VTable {
    lock: extern "C" fn(this: &OpaqueMutex),
    try_lock: extern "C" fn(this: &OpaqueMutex) -> bool,
    unlock: extern "C" fn(this: &OpaqueMutex),
    #[sabi(last_prefix_field)]
    try_lock_for: extern "C" fn(this: &OpaqueMutex, timeout: RDuration) -> bool,
}

impl VTable {
    const _TMP0: WithMetadata<VTable> = WithMetadata::new(VTable {
        lock,
        try_lock,
        unlock,
        try_lock_for,
    });

    // The VTABLE for this type in this executable/library
    const VTABLE: VTable_Ref = { VTable_Ref(Self::_TMP0.static_as_prefix()) };
}

extern "C" fn lock(this: &OpaqueMutex) {
    extern_fn_panic_handling! {
        this.value.lock();
    }
}
extern "C" fn try_lock(this: &OpaqueMutex) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock()
    }
}
extern "C" fn unlock(this: &OpaqueMutex) {
    extern_fn_panic_handling! {
        unsafe{
            this.value.unlock();
        }
    }
}
extern "C" fn try_lock_for(this: &OpaqueMutex, timeout: RDuration) -> bool {
    extern_fn_panic_handling! {
        this.value.try_lock_for(timeout.into())
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
        let mut mutex: RMutex<usize> = RMutex::new(0);
        *mutex.lock() += 100;
        *mutex.get_mut() += 100;
        *mutex.lock() += 100;
        assert_eq!(*mutex.lock(), 300);
    }

    #[test]
    fn into_inner() {
        let mutex: RMutex<usize> = RMutex::new(0);
        *mutex.lock() += 100;
        assert_eq!(mutex.into_inner(), 100);
    }

    #[test]
    fn debug_display() {
        let str_ = "\nhello\rhello\rhello\n";
        let mutex = RMutex::new(str_);
        let guard = mutex.lock();

        check_formatting_equivalence(&guard, str_);
    }

    #[cfg(miri)]
    const ITERS: usize = 10;

    #[cfg(not(miri))]
    const ITERS: usize = 0x1000;

    #[test]
    #[cfg(not(all(miri, target_os = "windows")))]
    fn lock() {
        static MUTEX: RMutex<usize> = RMutex::new(0);

        scoped_thread(|scope| {
            for _ in 0..8 {
                scope.spawn(move |_| {
                    for _ in 0..ITERS {
                        *MUTEX.lock() += 1;
                    }
                });
            }
        })
        .unwrap();

        assert_eq!(*MUTEX.lock(), 8 * ITERS);
    }

    #[test]
    #[cfg(not(all(miri, target_os = "windows")))]
    fn try_lock() {
        static MUTEX: RMutex<usize> = RMutex::new(0);

        scoped_thread(|scope| {
            for _ in 0..8 {
                scope.spawn(move |_| {
                    for _ in 0..ITERS {
                        loop {
                            if let RSome(mut guard) = MUTEX.try_lock() {
                                *guard += 1;
                                break;
                            }
                        }
                    }
                });
            }
        })
        .unwrap();

        scoped_thread(|scope| {
            let _guard = MUTEX.lock();
            scope.spawn(move |_| {
                assert_eq!(MUTEX.try_lock().map(drop), RNone);
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();

        assert_eq!(*MUTEX.lock(), 8 * ITERS);
    }

    #[test]
    #[cfg(not(all(miri, target_os = "windows")))]
    fn try_lock_for() {
        static MUTEX: RMutex<usize> = RMutex::new(0);

        scoped_thread(|scope| {
            for _ in 0..8 {
                scope.spawn(move |_| {
                    for i in 0..ITERS {
                        let wait_for = RDuration::new(0, (i as u32 + 1) * 500_000);
                        loop {
                            if let RSome(mut guard) = MUTEX.try_lock_for(wait_for) {
                                *guard += 1;
                                break;
                            }
                        }
                    }
                });
            }
        })
        .unwrap();

        #[cfg(not(miri))]
        scoped_thread(|scope| {
            let _guard = MUTEX.lock();
            scope.spawn(move |_| {
                assert_eq!(
                    MUTEX.try_lock_for(RDuration::new(0, 100_000)).map(drop),
                    RNone
                );
            });
            thread::sleep(Duration::from_millis(100));
        })
        .unwrap();

        assert_eq!(*MUTEX.lock(), 8 * ITERS);
    }
}
