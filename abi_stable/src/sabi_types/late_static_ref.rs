//! A late-initialized static reference.

use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

use crate::{
    external_types::RMutex,
    pointer_trait::{GetPointerKind, ImmutableRef, PK_Reference},
    prefix_type::{PrefixRef, PrefixRefTrait},
};

/// A late-initialized static reference,with fallible initialization.
///
/// As opposed to `Once`,
/// this allows initialization of its static reference to happen fallibly,
/// by returning a `Result<_,_>` from the `try_init` function,
/// or by panicking inside either initialization function.
///
/// On `Err(_)` and panics,one can try initialializing the static reference again.
///
/// # Example
///
/// This lazily loads a configuration file.
///
/// ```
///
/// use abi_stable::{
///     sabi_types::LateStaticRef,
///     std_types::{RBox, RBoxError, RHashMap, RString},
///     utils::leak_value,
/// };
///
/// use std::{fs, io, path::Path};
///
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// pub struct Config {
///     pub user_actions: RHashMap<RString, UserAction>,
/// }
///
/// #[derive(Deserialize)]
/// pub enum UserAction {
///     Include,
///     Ignore,
///     ReplaceWith,
/// }
///
/// fn load_config(file_path: &Path) -> Result<&'static Config, RBoxError> {
///     static CONFIG: LateStaticRef<&Config> = LateStaticRef::new();
///
///     CONFIG.try_init(|| {
///         let file = load_file(file_path).map_err(RBoxError::new)?;
///         let config =
///             serde_json::from_str::<Config>(&file).map_err(RBoxError::new)?;
///         Ok(leak_value(config))
///     })
/// }
///
/// # fn load_file(file_path:&Path)->Result<String,RBoxError>{
/// #     let str=r##"
/// #         {
/// #             "user_actions":{
/// #                 "oolong":"prolonged",
/// #                 "genius":"idiot"
/// #             }
/// #         }
/// #     "##.to_string();
/// #     Ok(str)
/// # }
///
/// ```
///
#[repr(C)]
#[derive(StableAbi)]
pub struct LateStaticRef<T> {
    pointer: AtomicPtr<()>,
    lock: RMutex<()>,
    _marker: PhantomData<T>,
}

#[allow(clippy::declare_interior_mutable_const)]
const LOCK: RMutex<()> = RMutex::new(());

unsafe impl<T: Sync> Sync for LateStaticRef<T> {}
unsafe impl<T: Send> Send for LateStaticRef<T> {}

impl<T> LateStaticRef<T> {
    /// Constructs the `LateStaticRef` in an uninitialized state.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::LateStaticRef;
    ///
    /// static LATE_REF: LateStaticRef<&String> = LateStaticRef::new();
    ///
    /// ```
    pub const fn new() -> Self {
        Self {
            lock: LOCK,
            pointer: AtomicPtr::new(ptr::null_mut()),
            _marker: PhantomData,
        }
    }
}

impl<T> LateStaticRef<&'static T> {
    /// Constructs `LateStaticRef`, initialized with `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::LateStaticRef;
    ///
    /// static LATE_REF: LateStaticRef<&&str> = LateStaticRef::from_ref(&"Hello!");
    ///
    /// ```
    pub const fn from_ref(value: &'static T) -> Self {
        Self {
            lock: LOCK,
            pointer: AtomicPtr::new(value as *const T as *mut ()),
            _marker: PhantomData,
        }
    }
}

impl<T> LateStaticRef<T> {
    /// Constructs `LateStaticRef` from a [`PrefixRef`].
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     pointer_trait::ImmutableRef,
    ///     prefix_type::{PrefixRefTrait, PrefixTypeTrait, WithMetadata},
    ///     sabi_types::LateStaticRef,
    ///     StableAbi,
    /// };
    ///
    /// fn main() {
    ///     assert_eq!(LATE_REF.get().unwrap().get_number()(), 100);
    /// }
    ///
    /// pub static LATE_REF: LateStaticRef<PersonMod_Ref> = {
    ///     // This is how you can construct a `LateStaticRef<Foo_Ref>`,
    ///     //  from a `Foo_Ref` at compile-time.
    ///     //
    ///     // If you don't need a `LateStaticRef` you can construct a `PersonMod_Ref` constant,
    ///     // and use that.
    ///     LateStaticRef::from_prefixref(MODULE.0)
    /// };
    ///
    /// #[repr(C)]
    /// #[derive(StableAbi)]
    /// #[sabi(kind(Prefix))]
    /// pub struct PersonMod {
    ///     /// The `#[sabi(last_prefix_field)]` attribute here means that this is
    ///     /// the last field in this struct that was defined in the
    ///     /// first compatible version of the library.
    ///     /// Moving this attribute is a braeking change.
    ///     #[sabi(last_prefix_field)]
    ///     pub get_number: extern "C" fn() -> u32,
    /// }
    ///
    /// const MODULE: PersonMod_Ref = {
    ///     const S: &WithMetadata<PersonMod> =
    ///         &WithMetadata::new(PersonMod { get_number });
    ///
    ///     PersonMod_Ref(S.static_as_prefix())
    /// };
    ///
    /// extern "C" fn get_number() -> u32 {
    ///     100
    /// }
    /// ```
    ///
    /// [`PrefixRef`]: ../prefix_type/struct.PrefixRef.html
    pub const fn from_prefixref(ptr: PrefixRef<T::PrefixFields>) -> Self
    where
        T: PrefixRefTrait + 'static,
        T::PrefixFields: 'static,
    {
        Self {
            lock: LOCK,
            pointer: AtomicPtr::new(ptr.to_raw_ptr() as *mut ()),
            _marker: PhantomData,
        }
    }
}

impl<T> LateStaticRef<T> {
    /// Constructs `LateStaticRef` from a `NonNull` pointer.
    ///
    /// # Safety
    ///
    /// The passed in pointer must be valid for passing to
    /// [`<T as ImmutableRef>::from_nonnull`],
    /// it must be a valid pointer to `U`,
    /// and be valid to dereference for the rest of the program's lifetime.
    ///
    /// [`<T as ImmutableRef>::from_nonnull`]:
    /// ../pointer_trait/trait.ImmutableRef.html#method.from_nonnull
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{
    ///     pointer_trait::{GetPointerKind, PK_Reference},
    ///     sabi_types::LateStaticRef,
    ///     utils::ref_as_nonnull,
    ///     StableAbi,
    /// };
    ///
    /// use std::ptr::NonNull;
    ///
    /// #[derive(Copy, Clone)]
    /// struct Foo<'a>(&'a u64);
    ///
    /// impl<'a> Foo<'a> {
    ///     const fn as_nonnull(self) -> NonNull<u64> {
    ///         ref_as_nonnull(self.0)
    ///     }
    /// }
    ///
    /// unsafe impl<'a> GetPointerKind for Foo<'a> {
    ///     type PtrTarget = u64;
    ///     type Kind = PK_Reference;
    /// }
    ///
    /// const MODULE: LateStaticRef<Foo<'static>> = {
    ///     unsafe {
    ///         LateStaticRef::from_custom(Foo(&100).as_nonnull())
    ///     }
    /// };
    /// ```
    pub const unsafe fn from_custom(ptr: NonNull<T::PtrTarget>) -> Self
    where
        T: GetPointerKind<Kind = PK_Reference> + 'static,
        T::PtrTarget: 'static,
    {
        Self {
            lock: LOCK,
            pointer: AtomicPtr::new(ptr.as_ptr() as *mut ()),
            _marker: PhantomData,
        }
    }
}

impl<T> LateStaticRef<T>
where
    T: ImmutableRef + 'static,
{
    /// Lazily initializes the `LateStaticRef` with `initializer`,
    /// returning the `T` if either it was already initialized,or
    /// if `initalizer` returned Ok(..).
    ///
    /// If `initializer` returns an `Err(...)` this returns the error and
    /// allows the `LateStaticRef` to be initializer later.
    ///
    /// If `initializer` panics,the panic is propagated,
    /// and the reference can be initalized later.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{sabi_types::LateStaticRef, utils::leak_value};
    ///
    /// static LATE: LateStaticRef<&String> = LateStaticRef::new();
    ///
    /// static EARLY: LateStaticRef<&&str> = LateStaticRef::from_ref(&"Hello!");
    ///
    /// assert_eq!(LATE.try_init(|| Err("oh no!")), Err("oh no!"));
    /// assert_eq!(
    ///     LATE.try_init(|| -> Result<&'static String, ()> {
    ///         Ok(leak_value("Yay".to_string()))
    ///     })
    ///     .map(|s| s.as_str()),
    ///     Ok("Yay"),
    /// );
    ///
    /// assert_eq!(EARLY.try_init(|| Err("oh no!")), Ok(&"Hello!"));
    ///
    ///
    /// ```
    pub fn try_init<F, E>(&self, initializer: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if let Some(pointer) = self.get() {
            return Ok(pointer);
        }

        let guard_ = self.lock.lock();

        if let Some(pointer) = self.get() {
            return Ok(pointer);
        }

        let pointer = initializer()?;

        self.pointer.store(
            pointer.to_raw_ptr() as *mut T::PtrTarget as *mut (),
            Ordering::Release,
        );

        drop(guard_);

        Ok(pointer)
    }

    /// Lazily initializes the `LateStaticRef` with `initializer`,
    /// returning the `T` if either it was already initialized,
    /// or `initalizer` returns it without panicking.
    ///
    /// If `initializer` panics,the panic is propagated,
    /// and the reference can be initalized later.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{sabi_types::LateStaticRef, utils::leak_value};
    ///
    /// static LATE: LateStaticRef<&String> = LateStaticRef::new();
    ///
    /// static EARLY: LateStaticRef<&&str> = LateStaticRef::from_ref(&"Hello!");
    ///
    /// let _ = std::panic::catch_unwind(|| {
    ///     LATE.init(|| panic!());
    /// });
    ///
    /// assert_eq!(LATE.init(|| leak_value("Yay".to_string())), &"Yay");
    ///
    /// assert_eq!(EARLY.init(|| panic!()), &"Hello!");
    ///
    /// ```
    #[inline]
    pub fn init<F>(&self, initializer: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.try_init(|| -> Result<T, std::convert::Infallible> { Ok(initializer()) })
            .expect("bug:LateStaticRef::try_init should only return an Err if `initializer` does")
    }

    /// Returns `Some(x:T)` if the `LateStaticRef` was initialized, otherwise returns `None`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{sabi_types::LateStaticRef, utils::leak_value};
    ///
    /// static LATE: LateStaticRef<&String> = LateStaticRef::new();
    ///
    /// static EARLY: LateStaticRef<&&str> = LateStaticRef::from_ref(&"Hello!");
    ///
    /// let _ = std::panic::catch_unwind(|| {
    ///     LATE.init(|| panic!());
    /// });
    ///
    /// assert_eq!(LATE.get(), None);
    /// LATE.init(|| leak_value("Yay".to_string()));
    /// assert_eq!(LATE.get().map(|s| s.as_str()), Some("Yay"));
    ///
    /// assert_eq!(EARLY.get(), Some(&"Hello!"));
    ///
    /// ```
    pub fn get(&self) -> Option<T> {
        unsafe { T::from_raw_ptr(self.pointer.load(Ordering::Acquire) as *const T::PtrTarget) }
    }
}

use ::std::panic::{RefUnwindSafe, UnwindSafe};

impl<T> UnwindSafe for LateStaticRef<T> {}
impl<T> RefUnwindSafe for LateStaticRef<T> {}

//////////////////////////////////////////////////////

//#[cfg(test)]
#[cfg(all(test, not(feature = "only_new_tests")))]
mod tests {
    use super::*;

    use std::panic::catch_unwind;

    static N_100: u32 = 100;
    static N_277: u32 = 277;

    #[test]
    fn test_init() {
        let ptr = LateStaticRef::<&u32>::new();

        assert_eq!(None, ptr.get());

        let caught = catch_unwind(|| {
            ptr.init(|| panic!());
        });
        assert!(caught.is_err());

        assert_eq!(None, ptr.get());

        assert_eq!(100, *ptr.init(|| &N_100));
        assert_eq!(100, *ptr.init(|| panic!("this should not run")));

        assert_eq!((&N_100) as *const u32, ptr.get().unwrap() as *const u32);
    }

    #[test]
    fn test_try_init() {
        let ptr = LateStaticRef::<&u32>::new();

        assert_eq!(None, ptr.get());

        let caught = catch_unwind(|| {
            let _ = ptr.try_init(|| -> Result<_, i32> { panic!() });
        });
        assert!(caught.is_err());

        assert_eq!(None, ptr.get());

        assert_eq!(Err(10), ptr.try_init(|| -> Result<_, i32> { Err(10) }));
        assert_eq!(Err(17), ptr.try_init(|| -> Result<_, i32> { Err(17) }));

        assert_eq!(Ok(&277), ptr.try_init(|| -> Result<_, i32> { Ok(&N_277) }));

        assert_eq!(
            Ok(&277),
            ptr.try_init(|| -> Result<_, i32> { panic!("this should not run") })
        );

        assert_eq!((&N_277) as *const u32, ptr.get().unwrap() as *const u32);
    }
}

////////////////////////////////////////////////////////////////////////////////
