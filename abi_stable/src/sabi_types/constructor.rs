use std::{
    cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
    fmt::{self, Debug, Display},
};

/// Newtype wrapper to pass function pointers to `const fn`.
///
/// A workaround for it not being possible to get a function pointer within a `const fn`,
/// since it's possible to pass structs that happen to have function pointer fields.
///
/// Every impl of this type delegates the impl to the return value of the wrapped function
/// (which it calls every time),don't use those impls if the function is likely expensive.
///
/// # Example
///
/// ```
/// use abi_stable::{
///     sabi_types::Constructor,
///     std_types::{RNone, ROption, RSome},
/// };
///
/// extern "C" fn returns_100() -> ROption<u32> {
///     RSome(100)
/// }
///
/// extern "C" fn returns_100b() -> ROption<u32> {
///     RSome(100)
/// }
///
/// extern "C" fn returns_200() -> ROption<u32> {
///     RSome(200)
/// }
///
/// extern "C" fn returns_none() -> ROption<u32> {
///     RNone
/// }
///
/// const A: Constructor<ROption<u32>> = Constructor(returns_100);
/// const B: Constructor<ROption<u32>> = Constructor(returns_100b);
/// const C: Constructor<ROption<u32>> = Constructor(returns_200);
/// const D: Constructor<ROption<u32>> = Constructor(returns_none);
///
/// assert_eq!(A, A);
/// assert_eq!(B, B);
/// assert_eq!(C, C);
/// assert_eq!(D, D);
///
/// assert_eq!(A, B);
///
/// assert_ne!(A, C);
/// assert_ne!(A, D);
/// assert_ne!(B, C);
/// assert_ne!(C, D);
///
/// ```
///
#[repr(transparent)]
#[derive(StableAbi)]
// #[sabi(debug_print)]
pub struct Constructor<T>(pub extern "C" fn() -> T);

impl<T> Copy for Constructor<T> {}

impl<T> Clone for Constructor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Debug for Constructor<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&self.get(), f)
    }
}

impl<T> Display for Constructor<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.get(), f)
    }
}

impl<T> Constructor<T> {
    /// Constructs a `T` by calling the wrapped function.
    pub fn get(self) -> T {
        (self.0)()
    }

    pub(crate) const fn wrap_slice(slice: &[extern "C" fn() -> T]) -> &[Constructor<T>] {
        unsafe { &*(slice as *const [extern "C" fn() -> T] as *const [Constructor<T>]) }
    }
    pub(crate) const fn unwrap_slice(slice: &[Constructor<T>]) -> &[extern "C" fn() -> T] {
        unsafe { &*(slice as *const [Constructor<T>] as *const [extern "C" fn() -> T]) }
    }
}

impl<T> Eq for Constructor<T> where T: Eq {}

impl<T> PartialEq for Constructor<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T> Ord for Constructor<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.get().cmp(&other.get())
    }
}

impl<T> PartialOrd for Constructor<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Either the constructor for a value or the value itself
#[repr(u8)]
#[derive(StableAbi, Copy, Clone)]
//#[sabi(debug_print)]
pub enum ConstructorOrValue<T> {
    /// This is an `extern "C" fn()->T` which is used to construct a value of type `T`
    Constructor(Constructor<T>),
    /// A value of type `T`
    Value(T),
}

impl<T> ConstructorOrValue<T> {
    /// Gets the wrapped value,computing it from its constructor if this
    /// is the `Constructor` variant
    pub fn get(&mut self) -> &T {
        match self {
            ConstructorOrValue::Value(v) => v,
            &mut ConstructorOrValue::Constructor(func) => {
                let v = (func.0)();
                *self = ConstructorOrValue::Value(v);
                match self {
                    ConstructorOrValue::Value(v) => v,
                    _ => unreachable!(),
                }
            }
        }
    }
}
