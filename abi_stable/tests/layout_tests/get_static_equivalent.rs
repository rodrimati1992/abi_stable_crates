use abi_stable::{
    abi_stability::{
        abi_checking::check_layout_compatibility,
        get_static_equivalent::{GetStaticEquivalent_, Unsized},
    },
    std_types::{RStr, RVec},
    type_layout::TypeLayout,
    StableAbi,
};

mod unit_type {
    use abi_stable::GetStaticEquivalent;

    #[derive(GetStaticEquivalent)]
    pub(super) struct Struct;

    impl super::UniqueId for Struct {
        const UID: i64 = -4_000_000;
    }
}

mod single_ty_param {
    use abi_stable::GetStaticEquivalent;

    #[derive(GetStaticEquivalent)]
    pub(super) struct Struct<T>(T);

    impl<T> super::UniqueId for Struct<T>
    where
        T: super::UniqueId,
    {
        const UID: i64 = 4_000_000_000_000i64 + T::UID;
    }
}

mod single_lt_param {
    use abi_stable::GetStaticEquivalent;

    #[derive(GetStaticEquivalent)]
    pub(super) struct Struct<'a>(&'a ());

    impl super::UniqueId for Struct<'_> {
        const UID: i64 = -1_000_000;
    }
}

mod single_lt_ty_param {
    use abi_stable::GetStaticEquivalent;

    #[derive(GetStaticEquivalent)]
    pub(super) struct Struct<'a, T>(&'a (), T);

    impl<T> super::UniqueId for Struct<'_, T>
    where
        T: super::UniqueId,
    {
        const UID: i64 = 1_000_000_000_000i64 + T::UID;
    }
}

mod sabi_with_0_ty_params {
    use abi_stable::StableAbi;

    #[repr(C)]
    #[derive(StableAbi)]
    pub(super) struct Struct;
}

#[cfg(not(feature = "no_fn_promotion"))]
mod sabi_with_1_ty_params {
    use super::UniqueId;
    use abi_stable::{marker_type::UnsafeIgnoredType, tag, StableAbi};

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(not_stableabi(T), bound = "T:UniqueId", tag = " tag!{ T::UID } ")]
    pub(super) struct Struct<T> {
        _inner: UnsafeIgnoredType<T>,
    }
}

#[cfg(not(feature = "no_fn_promotion"))]
mod sabi_with_2_ty_params {
    use super::UniqueId;
    use abi_stable::{marker_type::UnsafeIgnoredType, tag, StableAbi};

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        not_stableabi(T),
        not_stableabi(U),
        bound = "T:UniqueId",
        bound = "U:UniqueId",
        tag = " tag![[ tag!(T::UID) , tag!(U::UID) ]] "
    )]
    pub(super) struct Struct<T, U> {
        _inner: UnsafeIgnoredType<(T, U)>,
    }
}

trait UniqueId {
    const UID: i64;
}

macro_rules! declare_uids {
    ( $( ( $ty:ty, $index:expr ) )* ) => (
        $(
            impl<'a> UniqueId for $ty {
                const UID:i64=$index;
            }
        )*
    )
}

declare_uids! {
    ( &'a Unsized<str> , 1 )
    ( &'a Unsized<[u8]>, 3 )
    ( (), 7 )
    ( u64, 15 )
    ( RVec<u64>, 31 )
    ( RStr<'a>, 64 )
}

#[cfg(not(miri))]
fn type_layout_of<T>() -> &'static TypeLayout
where
    T: StableAbi,
{
    <T as StableAbi>::LAYOUT
}

#[cfg(not(feature = "no_fn_promotion"))]
#[cfg(not(miri))]
fn get_list_inner<T, U>() -> Vec<&'static TypeLayout>
where
    T: GetStaticEquivalent_ + UniqueId,
    U: GetStaticEquivalent_ + UniqueId,
{
    vec![
        type_layout_of::<sabi_with_1_ty_params::Struct<T>>(),
        type_layout_of::<sabi_with_2_ty_params::Struct<T, U>>(),
    ]
}

#[cfg(not(feature = "no_fn_promotion"))]
#[cfg(not(miri))]
fn get_list<'a, T>(_: &'a T) -> Vec<&'static TypeLayout> {
    type Ty0 = unit_type::Struct;
    type Ty1<'a> = single_ty_param::Struct<&'a Unsized<str>>;
    type Ty2<'a> = single_lt_param::Struct<'a>;
    type Ty3<'a> = single_lt_ty_param::Struct<'a, &'a Unsized<[u8]>>;
    type Ty4 = single_ty_param::Struct<()>;
    type Ty5 = single_ty_param::Struct<u64>;
    type Ty6 = single_ty_param::Struct<RVec<u64>>;
    type Ty7<'a> = single_ty_param::Struct<RStr<'a>>;

    vec![
        vec![type_layout_of::<sabi_with_0_ty_params::Struct>()],
        get_list_inner::<Ty0, Ty1<'a>>(),
        get_list_inner::<Ty1<'a>, Ty2<'a>>(),
        get_list_inner::<Ty2<'a>, Ty3<'a>>(),
        get_list_inner::<Ty3<'a>, Ty4>(),
        get_list_inner::<Ty4, Ty5>(),
        get_list_inner::<Ty5, Ty6>(),
        get_list_inner::<Ty6, Ty7<'a>>(),
        get_list_inner::<Ty7<'a>, Ty7<'a>>(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(not(feature = "no_fn_promotion"))]
#[cfg(not(miri))]
#[test]
fn check_not_stableabi() {
    let hello = Vec::<u32>::new();

    let list = get_list(&hello);

    for (interf_i, interf) in list.iter().cloned().enumerate() {
        for (impl_i, impl_) in list.iter().cloned().enumerate() {
            let res = check_layout_compatibility(interf, impl_);
            if interf_i == impl_i {
                assert_eq!(res, Ok(()));
            } else {
                assert_ne!(
                    res,
                    Ok(()),
                    "interf:\n{}\n\n\nimpl:\n{}\n\ninterf_i:{}   impl_i:{}\n\n",
                    interf,
                    impl_,
                    interf_i,
                    impl_i,
                );
            }
        }
    }
}
