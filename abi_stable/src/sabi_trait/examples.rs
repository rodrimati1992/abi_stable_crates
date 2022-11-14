use std::{fmt::Debug, hash::Hash, sync::Arc};

use core_extensions::SelfOps;

use crate::{
    sabi_trait::prelude::*,
    std_types::{RArc, RBox, ROption, RStr, RString, Tuple1},
    *,
};

//////////////////////////////////////

#[sabi_trait]
/// ```
/// use abi_stable::{
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// let _ = RSomething_TO::<_, (), u32>::from_value(RBox::new(10_u32), TD_Opaque);
///
/// ```
///
/// While RSomething_TO can be constructed from an RArc,
/// no method on the trait can be called because RSomething has mutable and by value methods.
///
/// ```compile_fail
/// use abi_stable::{
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// let what = RSomething_TO::from_ptr(RArc::new(100u32), TD_Opaque);
/// RSomething::into_value(what);
///
/// ```
///
///
/// Cannot create RSomething from a !Sync type.
/// ```compile_fail
/// use abi_stable::{
///     marker_type::*,
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// use std::marker::PhantomData;
///
/// let ptr = RBox::new(PhantomData::<UnsyncSend>);
/// let _ = RSomething_TO::<_, (), PhantomData<UnsyncSend>>::from_value(ptr, TD_Opaque);
///
/// ```
///
/// Cannot create RSomething from a !Send type.
/// ```compile_fail
/// use abi_stable::{
///     marker_type::*,
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// use std::marker::PhantomData;
///
/// let ptr = RBox::new(PhantomData::<SyncUnsend>);
/// let _ = RSomething_TO::<_, (), PhantomData<SyncUnsend>>::from_value(ptr, TD_Opaque);
///
/// ```
///
// #[sabi(debug_print_trait)]
pub trait RSomething<T>: Send + Sync + Clone + Debug {
    type Element: Debug;

    fn get(&self) -> &Self::Element;

    fn get_mut(&mut self) -> &mut Self::Element;

    #[sabi(last_prefix_field)]
    fn into_value(self) -> Self::Element;
}

macro_rules! impls_for_something {
    (
        $traitname:ident ,
        extra_bounds[ $($extra_bounds:tt)* ]
    ) => (

        impl $traitname<()> for u32{
            type Element=u32;

            fn get(&self)->&Self::Element{
                self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                self
            }
            fn into_value(self)->Self::Element{
                self
            }
        }

        impl<'a,T> $traitname<()> for &'a T
        where
            T:Send+Sync+Debug+$($extra_bounds)*
        {
            type Element=Self;

            fn get(&self)->&Self::Element{
                self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                self
            }
            fn into_value(self)->Self::Element{
                self
            }
        }

        impl<T> $traitname<()> for RBox<T>
        where
            T:Send+Sync+Debug+Clone+$($extra_bounds)*
        {
            type Element=T;

            fn get(&self)->&Self::Element{
                &**self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                &mut **self
            }
            fn into_value(self)->Self::Element{
                RBox::into_inner(self)
            }
        }


        impl<T> $traitname<()> for RArc<T>
        where
            T:Send+Sync+Debug+Clone+$($extra_bounds)*
        {
            type Element=T;

            fn get(&self)->&Self::Element{
                &**self
            }
            fn get_mut(&mut self)->&mut Self::Element{
                RArc::make_mut(self)
            }
            fn into_value(self)->Self::Element{
                (*self).clone()
            }
        }

    )
}

impls_for_something! { RSomething, extra_bounds[ Sized ] }
impls_for_something! { DSomething, extra_bounds[ Hash ] }

//////////////////////////////////////

#[sabi_trait]
//#[sabi(debug_print_trait)]
#[sabi(use_dyn_trait)]
pub trait DSomething<T>: Send + Sync + Clone + Debug + Hash {
    type Element: Debug;

    fn get(&self) -> &Self::Element;

    fn get_mut(&mut self) -> &mut Self::Element;

    #[sabi(last_prefix_field)]
    fn into_value(self) -> Self::Element;
}

//////////////////////////////////////

#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait EmptyTrait {}

impl EmptyTrait for () {}

impl EmptyTrait for u32 {}

impl<T> EmptyTrait for RArc<T> {}

impl<T> EmptyTrait for RBox<T> {}

//////////////////////////////////////

#[sabi_trait]
pub trait StaticTrait: 'static {}

//////////////////////////////////////

/// While RSomethingElse_TO can be constructed from an RArc,
/// no method on the trait can be called because RSomethingElse has mutable and by value methods.
///
/// ```compile_fail
/// use abi_stable::{
///     marker_type::*,
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// let what = RSomethingElse_TO::from_ptr(RArc::new(100u32), TD_Opaque);
/// RSomethingElse::into_value(what);
///
///
/// ```
///
///
/// ```
/// use abi_stable::{
///     marker_type::*,
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// use std::marker::PhantomData;
///
/// let ptr = RBox::new(PhantomData::<UnsyncSend>);
/// let _ = RSomethingElse_TO::from_value(ptr, TD_Opaque);
///
/// ```
///
/// Cannot create RSomethingElse from a !Send type.
/// ```compile_fail
/// use abi_stable::{
///     marker_type::*,
///     sabi_trait::{examples::*, prelude::*},
///     std_types::*,
/// };
///
/// use std::marker::PhantomData;
///
/// let ptr = RBox::new(PhantomData::<SyncUnsend>);
/// let _ = RSomethingElse_TO::from_value(ptr, TD_Opaque);
///
/// ```
pub struct Dummy0;

#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait RSomethingElse<T: Copy>: Send + Debug {
    fn get(&self) -> &T;

    #[sabi(last_prefix_field)]
    fn into_value(self) -> T;

    fn passthrough_string(&self, value: RString) -> RString {
        value
    }

    fn passthrough_arc(&self, value: RArc<u32>) -> RArc<u32> {
        value
    }
}

impl RSomethingElse<u32> for u32 {
    fn get(&self) -> &u32 {
        self
    }
    fn into_value(self) -> u32 {
        self
    }

    fn passthrough_string(&self, _: RString) -> RString {
        RString::new()
    }

    fn passthrough_arc(&self, _: RArc<u32>) -> RArc<u32> {
        RArc::new(77)
    }
}

impl<T> RSomethingElse<T> for RArc<T>
where
    T: Copy + Send + Sync + Debug,
{
    fn get(&self) -> &T {
        &**self
    }
    fn into_value(self) -> T {
        *self
    }
}

impl<T> RSomethingElse<T> for RBox<T>
where
    T: Copy + Send + Debug,
{
    fn get(&self) -> &T {
        &**self
    }
    fn into_value(self) -> T {
        *self
    }
}

//////////////////////////////////////

#[sabi_trait]
pub trait RFoo<'a, T: Copy + 'a> {
    fn get(&self) -> &'a T;
}

impl<'a> RFoo<'a, u32> for Tuple1<u32> {
    fn get(&self) -> &'a u32 {
        &10
    }
}

impl<'a, T> RFoo<'a, u64> for RArc<T> {
    fn get(&self) -> &'a u64 {
        &20
    }
}

//////////////////////////////////////

//////////////////////////////////////

#[sabi_trait]
//#[sabi(debug_print_trait)]
pub trait Dictionary {
    type Value;
    type Unused;
    fn what(&self, _: &Self::Unused);
    fn get(&self, key: RStr<'_>) -> Option<&Self::Value>;
    fn insert(&mut self, key: RString, value: Self::Value) -> ROption<Self::Value>;
}

//////////////////////////////////////

#[cfg(all(test))]
mod tests {
    use super::*;

    use crate::{
        sabi_types::{RMut, RRef},
        traits::IntoReprC,
    };

    fn assert_sync_send_debug_clone<T: Sync + Send + Debug + Clone>(_: &T) {}

    macro_rules! _something_test {
        (
            fn $fn_name:ident,
            fn $something_methods:ident,
            $typename:ident,
            $traitname:ident,
        ) => {
            #[test]
            fn $fn_name() {
                let number = 100_u32;
                let object = $typename::<_, (), u32>::from_value(number, TD_CanDowncast);
                let arcobj = $typename::<_, (), u32>::from_ptr(RArc::new(number), TD_CanDowncast);
                let erased = $typename::<_, (), u32>::from_ptr(RBox::new(number), TD_Opaque);

                assert_sync_send_debug_clone(&object);
                assert_sync_send_debug_clone(&arcobj);
                assert_sync_send_debug_clone(&erased);

                fn assertions_unerased(mut object: $typename<'_, RBox<()>, (), u32>) {
                    assert_eq!(object.obj.downcast_as::<u32>().ok(), Some(&100));
                    assert_eq!(object.obj.downcast_as::<i8>().ok(), None::<&i8>);
                    assert_eq!(object.obj.downcast_as_mut::<u32>().ok(), Some(&mut 100));
                    assert_eq!(object.obj.downcast_as_mut::<i8>().ok(), None::<&mut i8>);
                    object = object
                        .obj
                        .downcast_into::<i8>()
                        .unwrap_err()
                        .into_inner()
                        .piped($typename::from_sabi);
                    assert_eq!(object.obj.downcast_into::<u32>().ok(), Some(RBox::new(100)));
                }

                fn assertions_unerased_arc(mut object: $typename<'_, RArc<()>, (), u32>) {
                    assert_eq!(object.obj.downcast_as::<u32>().ok(), Some(&100));
                    assert_eq!(object.obj.downcast_as::<i8>().ok(), None::<&i8>);
                    object = object
                        .obj
                        .downcast_into::<i8>()
                        .unwrap_err()
                        .into_inner()
                        .piped($typename::from_sabi);
                    assert_eq!(object.obj.downcast_into::<u32>().ok(), Some(RArc::new(100)));
                }

                fn assertions_erased(mut object: $typename<'_, RBox<()>, (), u32>) {
                    assert_eq!(object.obj.downcast_as::<u32>().ok(), None);
                    assert_eq!(object.obj.downcast_as::<i8>().ok(), None);
                    assert_eq!(object.obj.downcast_as_mut::<u32>().ok(), None);
                    assert_eq!(object.obj.downcast_as_mut::<i8>().ok(), None);
                    object = object
                        .obj
                        .downcast_into::<u32>()
                        .unwrap_err()
                        .into_inner()
                        .piped($typename::from_sabi);
                    let _ = object.obj.downcast_into::<i8>().unwrap_err().into_inner();
                }

                fn create_from_ref<'a, T>(
                    value: &'a T,
                ) -> $typename<'a, RRef<'a, ()>, (), T::Element>
                where
                    T: $traitname<()> + 'a,
                {
                    $typename::<_, (), T::Element>::from_ptr(value, TD_Opaque)
                }

                fn create_from_val<'a, T>(value: T) -> $typename<'a, RBox<()>, (), T::Element>
                where
                    T: $traitname<()> + 'a,
                {
                    $typename::<_, (), T::Element>::from_value(value, TD_Opaque)
                }

                let what = RBox::new(100);
                let _ = create_from_ref(&*what);
                let _ = create_from_val(&*what);

                assert_eq!(format!("{:?}", number), format!("{:?}", erased));
                assert_eq!(format!("{:?}", number), format!("{:?}", arcobj));
                assert_eq!(format!("{:?}", number), format!("{:?}", object));

                assert_eq!(format!("{:#?}", number), format!("{:?}", erased));
                assert_eq!(format!("{:#?}", number), format!("{:?}", arcobj));
                assert_eq!(format!("{:#?}", number), format!("{:?}", object));

                assertions_unerased(object.clone());
                assertions_unerased(object);

                assertions_unerased_arc(arcobj.clone());
                assertions_unerased_arc(arcobj);

                assertions_erased(erased.clone());
                assertions_erased(erased);
            }

            #[test]
            fn $something_methods() {
                let mut object = $typename::<_, (), _>::from_value(100, TD_Opaque);
                let mut cloned = object.clone();

                assert_eq!(object.get(), &100);
                assert_eq!(object.get_mut(), &mut 100);
                assert_eq!(object.into_value(), 100);

                assert_eq!($traitname::get(&cloned), &100);
                assert_eq!($traitname::get_mut(&mut cloned), &mut 100);
                assert_eq!($traitname::into_value(cloned), 100);
            }
        };
    }

    _something_test! {
        fn construct_rsomething,
        fn rsomething_methods,
        RSomething_TO,
        RSomething,
    }

    _something_test! {
        fn construct_dsomething,
        fn dsomething_methods,
        DSomething_TO,
        DSomething,
    }

    #[test]
    fn construct_rempty() {
        let arc = Arc::new(107_u32);
        let rarc = arc.clone().into_c();

        assert_eq!(Arc::strong_count(&arc), 2);

        let mut object: EmptyTrait_TO<'_, RBox<()>> =
            EmptyTrait_TO::from_value(rarc.clone(), TD_CanDowncast);

        assert_eq!(Arc::strong_count(&arc), 3);

        let erased: EmptyTrait_TO<'_, RArc<()>> = EmptyTrait_TO::from_ptr(rarc.clone(), TD_Opaque);

        assert_eq!(Arc::strong_count(&arc), 4);

        assert_eq!(**object.obj.downcast_as::<RArc<u32>>().unwrap(), 107);
        assert_eq!(**object.obj.downcast_as_mut::<RArc<u32>>().unwrap(), 107);

        assert_eq!(Arc::strong_count(&arc), 4);
        object = object
            .obj
            .downcast_into::<u32>()
            .unwrap_err()
            .into_inner()
            .piped(EmptyTrait_TO::from_sabi);
        assert_eq!(Arc::strong_count(&arc), 4);

        assert_eq!(
            object.obj.downcast_into::<RArc<u32>>().unwrap(),
            RBox::new(RArc::new(107))
        );

        assert_eq!(Arc::strong_count(&arc), 3);

        erased.obj.downcast_into::<u32>().unwrap_err();

        assert_eq!(Arc::strong_count(&arc), 2);
    }

    #[test]
    fn test_reborrowing() {
        let arc = Arc::new(107_u32);
        let rarc = arc.clone().into_c();

        assert_eq!(Arc::strong_count(&arc), 2);

        let mut object: RSomething_TO<'_, RBox<()>, (), u32> =
            RSomething_TO::<_, (), u32>::from_value(rarc.clone(), TD_CanDowncast);

        assert_eq!(Arc::strong_count(&arc), 3);

        for _ in 0..10 {
            assert_eq!(
                object.obj.reborrow().downcast_into::<RArc<u32>>().unwrap(),
                RRef::new(&RArc::new(107))
            );
        }
        assert_eq!(Arc::strong_count(&arc), 3);

        for _ in 0..10 {
            assert_eq!(
                object
                    .obj
                    .reborrow_mut()
                    .downcast_into::<RArc<u32>>()
                    .unwrap(),
                RMut::new(&mut RArc::new(107))
            );
        }

        assert_eq!(Arc::strong_count(&arc), 3);

        {
            let cloned = object.obj.reborrow().clone();

            assert_eq!(format!("{:?}", cloned), "107");
        }

        assert_eq!(Arc::strong_count(&arc), 3);

        drop(object);

        assert_eq!(Arc::strong_count(&arc), 2);
    }

    #[test]
    fn rsomething_else() {
        {
            let object = RSomethingElse_TO::from_value(RArc::new(100_u32), TD_Opaque);
            let _: &dyn RSomethingElse<u32> = &object;

            assert_eq!(object.get(), &100);
            assert_eq!(object.passthrough_arc(RArc::new(90)), RArc::new(90));
            assert_eq!(
                object.passthrough_string(RString::from("what")),
                RString::from("what")
            );
            assert_eq!(object.into_value(), 100);
        }
        {
            let object = RSomethingElse_TO::from_value(RArc::new(100_u32), TD_Opaque);
            assert_eq!(RSomethingElse::get(&object,), &100);
            assert_eq!(
                RSomethingElse::passthrough_arc(&object, RArc::new(90)),
                RArc::new(90)
            );
            assert_eq!(
                RSomethingElse::passthrough_string(&object, RString::from("what")),
                RString::from("what")
            );
            assert_eq!(RSomethingElse::into_value(object), 100);
        }
        {
            let object = RSomethingElse_TO::<_, u32>::from_value(100u32, TD_CanDowncast);
            assert_eq!(
                RSomethingElse::passthrough_arc(&object, RArc::new(90)),
                RArc::new(77)
            );
            assert_eq!(
                RSomethingElse::passthrough_string(&object, RString::from("what")),
                RString::from("")
            );
        }
    }

    #[test]
    fn rfoo() {
        let object = &RFoo_TO::from_ptr(RBox::new(RArc::new(76)), TD_Opaque);
        let tuple1_object = &RFoo_TO::from_ptr(RArc::new(Tuple1(100)), TD_Opaque);

        assert_eq!(object.get(), &20);
        assert_eq!(tuple1_object.get(), &10);

        assert_eq!(RFoo::get(object), &20);
        assert_eq!(RFoo::get(tuple1_object), &10);
    }

    #[test]
    fn test_from_const() {
        const RS_U32: RSomething_CTO<'static, 'static, (), u32> =
            RSomething_CTO::from_const(&0, TD_Opaque);

        assert_eq!(RS_U32.get(), &0);

        fn make_const_rsomething<'borr, 'a, T, U>(ref_: &'a T) -> RSomething_CTO<'borr, 'a, (), U>
        where
            T: 'borr + RSomething<(), Element = U>,
            U: Debug,
        {
            RSomething_CTO::from_const(ref_, TD_Opaque)
        }

        let hi = make_const_rsomething(&77);
        assert_eq!(hi.get(), &77);
    }
}
