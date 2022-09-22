//! Contains the `GetStaticEquivalent_` trait and related items.

/// A type that stands in for `Self`,used to create a `UTypeId` for doing layout checking.
///
/// This may or may not have the same TypeId as Self.
///
/// # Safety
///
/// The `StaticEquivalent` associated type must be either of:
/// - the same type as `Self`, ignoring lifetime arguments.
/// - a type declared specifically to be the `StaticEquivalent`
/// associated type of `Self`(and no other type),
/// with the same type and const arguments as `Self`.
///
/// In either case, non-`'static` type parameters can be replaced with their
/// `GetStaticEquivalent_::StaticEquivalent` associated type.
///
pub unsafe trait GetStaticEquivalent_ {
    /// The `'static` equivalent of `Self`
    type StaticEquivalent: 'static;
}

/// Gets the `'static` equivalent of a type,only for use in creating a `UTypeId`.
pub type GetStaticEquivalent<T> = <T as GetStaticEquivalent_>::StaticEquivalent;

/// Used to avoid a `?Sized` bound on `GetStaticEquivalent_::StaticEquivalent`.
///
/// It's fine to use this instead of `str` and `[T]` since the type is
/// only required to be unique.
pub struct Unsized<T: ?Sized>(fn(&T));

////////////////////////////////////////////////////////////////////////////////
//                      Impls for non-StableAbi types
////////////////////////////////////////////////////////////////////////////////

unsafe impl GetStaticEquivalent_ for str {
    type StaticEquivalent = Unsized<str>;
}

unsafe impl<T> GetStaticEquivalent_ for [T]
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = Unsized<[T::StaticEquivalent]>;
}

unsafe impl<T: ?Sized> GetStaticEquivalent_ for Unsized<T>
where
    T: GetStaticEquivalent_,
{
    type StaticEquivalent = Unsized<T::StaticEquivalent>;
}
