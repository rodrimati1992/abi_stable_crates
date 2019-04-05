macro_rules! declare_struct {
    (
        manual =$manual:ident
        derived=$derived:ident

        $( #[$meta:meta] )*
        struct [$($generics:tt)*] {
            $($struct_contents:tt)*
        }
    ) => (
        $( #[$meta] )*
        pub struct $manual<$($generics)*>{
            $($struct_contents)*
        }

        $( #[$meta] )*
        #[derive(StableAbi)]
        pub struct $derived<$($generics)*>{
            $($struct_contents)*
        }
    )
}


/////////////////////////////////////////////////////////
////      repr(C) struct
/////////////////////////////////////////////////////////


declare_struct!{
    manual =PointManual
    derived=Point


    #[repr(C)]
    struct [T] {
        x:T,
        y:T,
    }
}


unsafe impl<T> _sabi_reexports::MaybeStableAbi for Point
where
    T:StableAbi,
    T:StableAbi,
{
    type Kind = _sabi_reexports::Value_Kind;
    type IsNonZeroType = _sabi_reexports::False;
    const LAYOUT: &'static _sabi_reexports::TypeLayout = {
        let params=&_sabi_reexports::TypeLayoutParams {
            name: "Point",
            package:StaticStr::new(env!("CARGO_PKG_NAME")),
            package_version:_sabi_reexports::VersionStrings{
                major:StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                minor:StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                patch:StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
            },
            data: _sabi_reexports::TLData::Struct{
                fields:StaticSlice::new(&[
                    TLField::new("x",&[],<T as StableAbi>::ABI_INFO.get()),
                    TLField::new("y",&[],<T as StableAbi>::ABI_INFO.get()),
                ])
            },
            generics: tl_genparams!(;T;),
            phantom_fields: &[],
        };
        &_sabi_reexports::TypeLayout::from_params::<Self>(params)
    };
}

/////////////////////////////////////////////////////////
////      repr(C) enum
/////////////////////////////////////////////////////////


macro_rules! declare_direction {
    (
        $( #[$meta:meta] )*
        enum $direction:ident;
    ) => (
        $( #[$meta] )*
        #[repr(C)]
        pub enum $direction<T>
        where
            T:Copy
        {
            Left,
            Other{
                name:&'static str,
                other:T,
            },
            Right,
        }
    )
}

declare_direction!(
    enum DirectionManualImpl;
)

declare_direction!(
    #[derive(StableAbi)]
    enum Direction;
)


/// Used to check that removing a variant causes a runtime type error.
#[derive(StableAbi)]
#[repr(C)]
pub enum DirectionRemovedOther<T>
where
    T:Copy
{
    Left,
    Right,
}


unsafe impl<T> _sabi_reexports::MaybeStableAbi for DirectionManualImpl<T>
where
    T:Copy,
    T:StableAbi,
{
    type Kind = _sabi_reexports::Value_Kind;
    type IsNonZeroType = _sabi_reexports::False;
    const LAYOUT: &'static _sabi_reexports::TypeLayout = {
        let params=&_sabi_reexports::TypeLayoutParams {
            name: "Direction",
            package:StaticStr::new(env!("CARGO_PKG_NAME")),
            package_version:_sabi_reexports::VersionStrings{
                major:StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                minor:StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                patch:StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
            },
            data: _sabi_reexports::TLData::enum_(&[
                TLEnumVariant::new("Left",&[]),
                TLEnumVariant::new("Other",&[
                    TLField::new(
                        "name",
                        &[LifetimeIndex::Static],
                        <&'static str as StableAbi>::ABI_INFO.get(),
                    ),
                    TLField::new(
                        "other",
                        &[],
                        <T as StableAbi>::ABI_INFO.get(),
                    ),
                ]),
                TLEnumVariant::new("Right",&[]),
            ]),
            generics: tl_genparams!(;T;),
            phantom_fields: &[],
        };
        &_sabi_reexports::TypeLayout::from_params::<Self>(params)
    };
}



/////////////////////////////////////////////////////////
////      repr(transparent) struct
/////////////////////////////////////////////////////////


#[derive(StableAbi)]
#[sabi(kind(unsafe_Prefix))]
#[repr(transparent)]
pub struct Name<'a,T>(&'a str,(),PhantomData<T>);


unsafe impl<'a> _sabi_reexports::MaybeStableAbi for Name<'a>
where
    &'a str:StableAbi,
    ():StableAbi,
    PhantomData<T>:StableAbi,
{
    type Kind = _sabi_reexports::Prefix_Kind;
    type IsNonZeroType = _sabi_reexports::False;
    const LAYOUT: &'static _sabi_reexports::TypeLayout = {
        let params=&_sabi_reexports::TypeLayoutParams {
            name: "Name",
            package:StaticStr::new(env!("CARGO_PKG_NAME")),
            package_version:_sabi_reexports::VersionStrings{
                major:StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                minor:StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                patch:StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
            },
            data: _sabi_reexports::TLData::ReprTransparent(
                <&'static str as StableAbi>::ABI_INFO.get()
            ),
            generics: tl_genparams!('a;T;),
            phantom_fields: &[
                TLField::new(
                    "0",
                    &[LifetimeIndex::Param(0)],
                    <&'static str as StableAbi>::ABI_INFO.get()
                ),
                TLField::new("1",&[],<() as StableAbi>::ABI_INFO.get()),
                TLField::new("2",&[],<PhantomData<T> as StableAbi>::ABI_INFO.get()),
            ],
        };
        &_sabi_reexports::TypeLayout::from_params::<Self>(params)
    };
}

