use std::{
    fmt::{self, Display},
    marker::PhantomData,
    ptr::NonNull,
};

use crate::{
    pointer_trait::{AsMutPtr, AsPtr, CanTransmuteElement, GetPointerKind, PK_MutReference},
    sabi_types::RRef,
};

/// Equivalent to `&'a mut T`,
/// which allows a few more operations without causing Undefined Behavior.
///
/// # Purpose
///
/// This type is used as the `&mut self` parameter in abi_stable trait objects
/// because it can be soundly transmuted
/// to point to other smaller but compatible types, then back to the original type.
///
/// This crate is tested with [miri] to detect bugs in unsafe code,
/// which implements the  [Stacked Borrows model].
/// Because that model forbids `&mut T` to `&mut ()`  to `&mut T` transmutes
/// (when `T` isn't zero-sized),
/// it required defining `RMut` to allow a mutable-reference-like type that can be transmuted.
///
/// # Example
///
/// This example demonstrates how a simple `&mut dyn Any`-like type can be implemented.
///
/// ```rust
/// use abi_stable::{marker_type::ErasedObject, std_types::UTypeId, RMut};
///
/// fn main() {
///     let mut value = WithTypeId::new(5u32);
///     let mut clone = value.clone();
///     let mut erased = value.erase();
///
///     assert_eq!(WithTypeId::downcast::<i32>(erased.reborrow()), None);
///     assert_eq!(WithTypeId::downcast::<bool>(erased.reborrow()), None);
///     assert_eq!(
///         WithTypeId::downcast::<u32>(erased.reborrow()),
///         Some(&mut clone)
///     );
/// }
///
/// // `#[repr(C))]` with a trailing `T` field is required for soundly transmuting from
/// // `RMut<'a, WithTypeId<T>>` to `RMut<'a, WithTypeId<ErasedObject>>`.
/// #[repr(C)]
/// #[derive(Debug, PartialEq, Clone)]
/// struct WithTypeId<T> {
///     type_id: UTypeId,
///     value: T,
/// }
///
/// impl<T> WithTypeId<T> {
///     pub fn new(value: T) -> Self
///     where
///         T: 'static,
///     {
///         Self {
///             type_id: UTypeId::new::<T>(),
///             value,
///         }
///     }
///
///     pub fn erase(&mut self) -> RMut<'_, WithTypeId<ErasedObject>> {
///         unsafe { RMut::new(self).transmute::<WithTypeId<ErasedObject>>() }
///     }
/// }
///
/// impl WithTypeId<ErasedObject> {
///     pub fn downcast<T>(this: RMut<'_, Self>) -> Option<&mut WithTypeId<T>>
///     where
///         T: 'static,
///     {
///         if this.get().type_id == UTypeId::new::<T>() {
///             // safety: we checked that type parameter was `T`
///             unsafe { Some(this.transmute_into_mut::<WithTypeId<T>>()) }
///         } else {
///             None
///         }
///     }
/// }
///
///
/// ```
///
/// <span id="type-prefix-exp"></span>
/// # Type Prefix
///
/// A type parameter `U` is considered a prefix of `T` in all of these cases:
///
/// - `U` is a zero-sized type with an alignment equal or lower than `T`
///
/// - `U` is a `#[repr(transparent)]` wrapper over `T`
///
/// - `U` and `T` are both `#[repr(C)]` structs,
/// in which `T` starts with the fields of `U` in the same order,
/// and `U` has an alignment equal to or lower than `T`.
///
/// Please note that it can be unsound to transmute a non-local
/// type if it has private fields,
/// since it may assume it was constructed in a particular way.
///
/// [Stacked Borrows model]:
/// https://github.com/rust-lang/unsafe-code-guidelines/blob/master/wip/stacked-borrows.md
///
/// [miri]: https://github.com/rust-lang/miri
///
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound(T:'a))]
pub struct RMut<'a, T> {
    ref_: NonNull<T>,
    _marker: PhantomData<crate::utils::MutRef<'a, T>>,
}

impl<'a, T> Display for RMut<'a, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.get(), f)
    }
}

unsafe impl<'a, T> Sync for RMut<'a, T> where &'a T: Sync {}

unsafe impl<'a, T> Send for RMut<'a, T> where &'a T: Send {}

shared_impls! {
    mod=static_ref_impls
    new_type=RMut['a][T],
    original_type=AAAA,
    deref_approach=(method = get),
}

impl<'a, T> RMut<'a, T> {
    /// Constructs this RMut from a mutable reference
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut foo = 3;
    /// let mut rmut = RMut::new(&mut foo);
    /// *rmut.get_mut() += 10;
    ///
    /// assert_eq!(*rmut.get(), 13);
    /// assert_eq!(foo, 13);
    ///
    /// ```
    #[inline(always)]
    pub fn new(ref_: &'a mut T) -> Self {
        unsafe {
            Self {
                ref_: NonNull::new_unchecked(ref_),
                _marker: PhantomData,
            }
        }
    }

    /// Constructs this RMut from a raw pointer.
    ///
    /// # Safety
    ///
    /// You must ensure that the raw pointer is valid for the `'a` lifetime,
    /// points to a fully initialized and aligned `T`,
    /// and that this is the only active pointer to that value.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut foo = 3u32;
    /// // safety:
    /// // `&mut foo` is casted to a pointer to a compatible type (`u32` to `i32`),
    /// // `rmut` is only used for the lifetime of foo,
    /// // and is the only active pointer to `foo` while it's used.
    /// let mut rmut = unsafe { RMut::from_raw((&mut foo) as *mut u32 as *mut i32) };
    /// *rmut.get_mut() -= 4;
    ///
    /// assert_eq!(*rmut.get(), -1);
    /// assert_eq!(foo, !0);
    ///
    /// ```
    #[inline(always)]
    pub const unsafe fn from_raw(ref_: *mut T) -> Self
    where
        T: 'a,
    {
        Self {
            ref_: unsafe { NonNull::new_unchecked(ref_) },
            _marker: PhantomData,
        }
    }

    /// Reborrows this `RMut`, with a shorter lifetime.
    ///
    /// This allows passing an `RMut` to functions multiple times,
    /// but with a shorter lifetime argument.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut foo = 3;
    /// let mut rmut = RMut::new(&mut foo);
    ///
    /// assert_eq!(mutate(rmut.reborrow()), 6);
    /// assert_eq!(mutate(rmut.reborrow()), 12);
    /// assert_eq!(mutate(rmut.reborrow()), 24);
    ///
    /// // last use of rmut, so it can be moved instead of being reborrowed.
    /// assert_eq!(mutate(rmut), 48);
    ///
    /// fn mutate(mut rmut: RMut<'_, u32>) -> u32 {
    ///     *rmut.get_mut() *= 2;
    ///     rmut.get_copy()
    /// }
    ///
    /// ```
    #[inline(always)]
    pub fn reborrow(&mut self) -> RMut<'_, T> {
        RMut {
            ref_: self.ref_,
            _marker: PhantomData,
        }
    }

    /// Reborrows this `RMut` into a shared reference.
    ///
    /// Note that because the reference reborrows this `RMut<'a, T>`
    /// its lifetime argument is strictly smaller.
    /// To turn an `RMut<'a, T>` into a `&'a T` (with the same lifetime argument)
    /// you can use [`into_ref`](#method.into_ref).
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 89;
    /// let rmut = RMut::new(&mut val);
    ///
    /// assert_eq!(rmut.get(), &89);
    ///
    /// ```
    ///
    /// ### Lifetimes
    ///
    /// This demonstrates when `into_ref` works, but `get` doesn't.
    ///
    /// ```rust
    /// # use abi_stable::RMut;
    /// fn stuff<'a>(x: RMut<'a, i32>) -> &'a i32 {
    ///     x.into_ref()
    /// }
    /// ```
    ///
    /// This doesn't compile, because `get` reborrows `foo`.
    /// ```compile_fail
    /// # use abi_stable::RMut;
    /// fn stuff<'a>(foo: RMut<'a, i32>) -> &'a i32 {
    ///     foo.get()
    /// }
    /// ```
    #[inline(always)]
    pub const fn get(&self) -> &T {
        unsafe { crate::utils::deref!(self.ref_.as_ptr()) }
    }

    /// Copies the value that this `RMut` points to.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = "hello";
    /// let mut rmut = RMut::new(&mut val);
    ///
    /// *rmut.get_mut() = "world";
    ///
    /// assert_eq!(rmut.get_copy(), "world");
    ///
    /// ```
    #[inline(always)]
    pub const fn get_copy(&self) -> T
    where
        T: Copy,
    {
        unsafe { *(self.ref_.as_ptr() as *const T) }
    }

    /// Converts this `RMut<'a, T>` into a `&'a T`
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 89;
    ///
    /// assert_eq!(mutate(RMut::new(&mut val)), &44);
    ///
    /// fn mutate(mut rmut: RMut<'_, u32>) -> &u32 {
    ///     *rmut.get_mut() /= 2;
    ///     rmut.into_ref()
    /// }
    ///
    /// ```
    ///
    #[inline(always)]
    pub const fn into_ref(self) -> &'a T {
        unsafe { crate::utils::deref!(self.ref_.as_ptr()) }
    }

    /// Reborrows this `RMut` into a mutable reference.
    ///
    /// Note that because the mutable reference reborrows this `RMut<'a, T>`
    /// its lifetime argument is strictly smaller.
    /// To turn an `RMut<'a, T>` into a `&'a mut T` (with the same lifetime argument)
    /// you can use [`into_mut`](#method.into_mut).
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 89;
    /// let mut rmut = RMut::new(&mut val);
    ///
    /// assert_eq!(rmut.get_mut(), &mut 89);
    ///
    /// *rmut.get_mut() += 10;
    ///
    /// assert_eq!(rmut.get_mut(), &mut 99);
    ///
    /// ```
    ///
    /// ### Lifetimes
    ///
    /// This demonstrates when `into_mut` works, but `get_mut` doesn't.
    ///
    /// ```rust
    /// # use abi_stable::RMut;
    /// fn stuff<'a>(x: RMut<'a, i32>) -> &'a mut i32 {
    ///     x.into_mut()
    /// }
    /// ```
    ///
    /// This doesn't compile, because `get_mut` reborrows `foo`.
    /// ```compile_fail
    /// # use abi_stable::RMut;
    /// fn stuff<'a>(mut foo: RMut<'a, i32>) -> &'a mut i32 {
    ///     foo.get_mut()
    /// }
    /// ```
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ref_.as_ptr() }
    }

    /// Converts this `RMut<'a, T>` into a `&'a mut T`
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 13;
    ///
    /// let rmut = RMut::new(&mut val);
    ///
    /// assert_eq!(rmut.get(), &13);
    ///
    /// *rmut.into_mut() += 8;
    ///
    /// assert_eq!(val, 21);
    ///
    /// ```
    #[inline(always)]
    pub fn into_mut(self) -> &'a mut T {
        unsafe { &mut *self.ref_.as_ptr() }
    }

    /// Reborrows this `RMut` as a const raw pointer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 34;
    /// let rmut = RMut::new(&mut val);
    ///
    /// unsafe {
    ///     assert_eq!(*rmut.as_ptr(), 34);
    /// }
    /// ```
    #[inline]
    pub const fn as_ptr(&self) -> *const T {
        self.ref_.as_ptr()
    }

    /// Reborrows this `RMut` as a mutable raw pointer.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 34;
    /// let mut rmut = RMut::new(&mut val);
    ///
    /// unsafe {
    ///     rmut.as_mut_ptr().write(7);
    ///
    ///     *rmut.as_mut_ptr() *= 2;
    ///
    ///     assert_eq!(val, 14);
    /// }
    /// ```
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ref_.as_ptr()
    }

    /// Converts this `RMut<'a, T>` into a `*mut T`
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 89;
    /// let rmut = RMut::new(&mut val);
    ///
    /// unsafe {
    ///     let ptr = rmut.into_raw();
    ///
    ///     ptr.write(27);
    ///
    ///     *ptr += 2;
    ///
    ///     assert_eq!(val, 29);
    /// }
    /// ```
    #[inline]
    pub const fn into_raw(self) -> *mut T {
        self.ref_.as_ptr()
    }

    /// Transmutes this `RMut<'a, T>` to a `*mut U`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = Direction::Up;
    /// let rmut = RMut::new(&mut val);
    ///
    /// assert_eq!(rmut.get(), &Direction::Up);
    ///
    /// let ptr = rmut.transmute_into_raw::<u8>();
    ///
    /// unsafe {
    ///     assert_eq!(*ptr, 2);
    ///     *ptr = 3;
    /// }
    ///
    /// assert_eq!(val, Direction::Down);
    ///
    /// #[repr(u8)]
    /// #[derive(Debug, PartialEq)]
    /// enum Direction {
    ///     Left = 0,
    ///     Right = 1,
    ///     Up = 2,
    ///     Down = 3,
    /// }
    ///
    /// ```
    #[inline(always)]
    pub const fn transmute_into_raw<U>(self) -> *mut U {
        self.ref_.as_ptr() as *mut U
    }

    /// Transmutes this `RMut<'a, T>` to a `&'a mut U`.
    ///
    /// # Safety
    ///
    /// Either of these must be the case:
    ///
    /// - [`U` is a prefix of `T`](#type-prefix-exp)
    ///
    /// - `RMut<'a, U>` was the original type of this `RMut<'a, T>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val = 13u8;
    /// let rmut = RMut::new(&mut val);
    ///
    /// assert_eq!(rmut.get(), &13);
    ///
    /// unsafe {
    ///     *rmut.transmute_into_mut::<i8>() = -1;
    /// }
    ///
    /// assert_eq!(val, 255);
    ///
    /// ```
    #[inline(always)]
    pub unsafe fn transmute_into_mut<U>(self) -> &'a mut U
    where
        U: 'a,
    {
        unsafe { &mut *(self.ref_.as_ptr() as *mut U) }
    }

    /// Transmutes this `RMut<'a, T>` to a `RMut<'a,U>`.
    ///
    /// # Safety
    ///
    /// Either of these must be the case:
    ///
    /// - [`U` is a prefix of `T`](#type-prefix-exp)
    ///
    /// - `RMut<'a, U>` was the original type of this `RMut<'a, T>`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::RMut;
    ///
    /// let mut val: [u32; 3] = [2, 3, 0];
    /// let mut rmut = RMut::new(&mut val);
    ///
    /// unsafe {
    ///     // safety:
    ///     // it's sound to transmute mutable references of arrays into shorter arrays.
    ///     //
    ///     // The `.reborrow()` prevents the `rmut` from being consumed.
    ///     compute_next(rmut.reborrow().transmute::<[u32; 2]>());
    ///     assert_eq!(rmut.get_copy(), [3, 5, 0]);
    ///
    ///     compute_next(rmut.reborrow().transmute::<[u32; 2]>());
    ///     assert_eq!(rmut.get_copy(), [5, 8, 0]);
    ///
    ///     // last use of `rmut`, so no need to reborrow
    ///     compute_next(rmut.transmute::<[u32; 2]>());
    /// }
    ///
    /// assert_eq!(val, [8, 13, 0]);
    ///
    /// fn compute_next(rmut: RMut<'_, [u32; 2]>) {
    ///     let [v0, v1] = rmut.into_mut();
    ///     let next = *v0 + *v1;
    ///     *v0 = std::mem::replace(v1, next);
    /// }
    /// ```
    #[inline(always)]
    pub const unsafe fn transmute<U>(self) -> RMut<'a, U>
    where
        U: 'a,
    {
        unsafe { RMut::from_raw(self.ref_.as_ptr() as *mut U) }
    }

    /// Reborrows this `RMut<'a, T>` into an `RRef<'_, T>`
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{RMut, RRef};
    ///
    /// let mut val = 77;
    /// let rmut = RMut::new(&mut val);
    ///
    /// for _ in 0..10 {
    ///     assertion(rmut.as_rref());
    /// }
    ///
    /// fn assertion(rref: RRef<'_, u32>) {
    ///     assert_eq!(rref.get_copy(), 77);
    /// }
    /// ```
    #[inline(always)]
    #[allow(clippy::needless_lifetimes)]
    pub const fn as_rref<'r>(&'r self) -> RRef<'r, T> {
        unsafe { RRef::from_raw(self.ref_.as_ptr()) }
    }

    /// Converts this `RMut<'a, T>` to an `RRef<'_, T>`
    ///
    /// # Example
    ///
    /// ```rust
    /// use abi_stable::{RMut, RRef};
    ///
    /// let mut val = 0;
    /// let rmut = RMut::new(&mut val);
    ///
    /// assertion(rmut.into_rref());
    ///
    /// fn assertion(rref: RRef<'_, u32>) {
    ///     assert_eq!(rref.get_copy(), 0);
    /// }
    /// ```
    #[inline(always)]
    pub const fn into_rref(self) -> RRef<'a, T> {
        unsafe { RRef::from_raw(self.ref_.as_ptr()) }
    }
}

unsafe impl<'a, T> AsPtr for RMut<'a, T> {
    #[inline(always)]
    fn as_ptr(&self) -> *const T {
        self.ref_.as_ptr() as *const T
    }
    #[inline(always)]
    fn as_rref(&self) -> RRef<'_, T> {
        unsafe { RRef::from_raw(self.ref_.as_ptr() as *const T) }
    }
}

unsafe impl<'a, T> AsMutPtr for RMut<'a, T> {
    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut T {
        self.ref_.as_ptr()
    }

    #[inline(always)]
    fn as_rmut(&mut self) -> RMut<'_, T> {
        self.reborrow()
    }
}

unsafe impl<'a, T> GetPointerKind for RMut<'a, T> {
    type Kind = PK_MutReference;

    type PtrTarget = T;
}

unsafe impl<'a, T, U> CanTransmuteElement<U> for RMut<'a, T>
where
    U: 'a,
{
    type TransmutedPtr = RMut<'a, U>;

    #[inline(always)]
    unsafe fn transmute_element_(self) -> Self::TransmutedPtr {
        unsafe { self.transmute() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_test() {
        unsafe {
            let val: *mut i32 = &mut 3;
            let mut rmut = RMut::from_raw(val);
            *rmut.get_mut() += 5;
            assert_eq!(rmut.get_copy(), 8);
        }
        {
            let val = &mut 3;
            let mut rmut = RMut::new(val);
            *rmut.get_mut() += 5;
            assert_eq!(rmut.get_copy(), 8);
        }
    }

    #[test]
    fn access() {
        let mut num = 5;
        let mut mutref = RMut::new(&mut num);

        assert_eq!(*mutref.get_mut(), 5);
        *mutref.get_mut() = 21;

        assert_eq!(*mutref.get(), 21);
        assert_eq!(mutref.get_copy(), 21);
        assert_eq!(*mutref.reborrow().into_ref(), 21);

        *mutref.reborrow().into_mut() = 34;

        unsafe {
            assert_eq!(*mutref.as_ptr(), 34);

            *mutref.as_mut_ptr() = 55;

            assert_eq!(*mutref.reborrow().into_raw(), 55);
            *mutref.reborrow().into_raw() = 89;
        }
        assert_eq!(num, 89);
    }

    #[test]
    fn transmutes() {
        let mut num = 0u8;

        unsafe {
            let ptr = RMut::new(&mut num).transmute_into_raw::<Enum>();
            assert_eq!(*ptr, Enum::Foo);

            ptr.write(Enum::Bar);
            assert_eq!(*ptr, Enum::Bar);
            assert_eq!(num, 1);
        }
        unsafe {
            let mref = RMut::new(&mut num).transmute_into_mut::<Enum>();
            assert_eq!(*mref, Enum::Bar);

            *mref = Enum::Qux;
            assert_eq!(*mref, Enum::Qux);
            assert_eq!(num, 2);
        }
        unsafe {
            let mut rmut = RMut::new(&mut num).transmute::<Enum>();
            assert_eq!(rmut, RMut::new(&mut Enum::Qux));

            *rmut.get_mut() = Enum::Foo;
            assert_eq!(*rmut.get(), Enum::Foo);
            assert_eq!(num, 0);
        }
        unsafe {
            let mut rmut: RMut<'_, Enum> = RMut::new(&mut num).transmute_element_();
            assert_eq!(rmut, RMut::new(&mut Enum::Foo));

            *rmut.get_mut() = Enum::Bar;
            assert_eq!(*rmut.get(), Enum::Bar);
            assert_eq!(num, 1);
        }
    }

    #[test]
    fn as_rtype_test() {
        let mut num = 0u8;
        let mut rmut = RMut::new(&mut num);

        assert_eq!(rmut.as_rref(), RRef::new(&0));
        assert_eq!(rmut.reborrow().into_rref(), RRef::new(&0));

        assert_eq!(rmut.as_rmut(), RMut::new(&mut 0));
    }

    #[derive(Debug, PartialEq)]
    #[repr(u8)]
    enum Enum {
        Foo = 0,
        Bar = 1,
        Qux = 2,
    }
}
