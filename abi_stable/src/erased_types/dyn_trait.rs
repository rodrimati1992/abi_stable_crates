/*!
Contains the `DynTrait` type,and related traits/type aliases.
*/

use std::{
    fmt::{self,Write as fmtWrite},
    io,
    ops::DerefMut,
    marker::PhantomData,
    mem::ManuallyDrop,
    ptr,
    rc::Rc,
};

use serde::{de, ser, Deserialize, Deserializer};

#[allow(unused_imports)]
use core_extensions::{prelude::*, ResultLike};

use crate::{
    abi_stability::SharedStableAbi,
    pointer_trait::{
        StableDeref, TransmuteElement,
        GetPointerKind,PK_SmartPointer,PK_Reference,
    },
    marker_type::{ErasedObject,UnsafeIgnoredType}, 
    std_types::{RBox, RCow, RStr,RVec,RIoError},
};

#[allow(unused_imports)]
use crate::std_types::Tuple2;

use super::*;
use super::{
    c_functions::adapt_std_fmt,
    trait_objects::*,
    vtable::{GetVtable, VTable},
    traits::InterfaceFor,
};


#[cfg(all(test,not(feature="only_new_tests")))]
mod tests;

mod priv_ {
    use super::*;


    /**

DynTrait implements ffi-safe trait objects,for a selection of traits.

# Passing opaque values around with `DynTrait<_>`

One can pass non-StableAbi types around by using type erasure,using this type.

It generally looks like `DynTrait<'borrow,Pointer<()>,Interface>`,where:

- `'borrow` is the borrow that the type that was erased had.

- `Pointer` is some `pointer_trait::StableDeref` pointer type.

- `Interface` is an `InterfaceType`,which describes what traits are 
    required when constructing the `DynTrait<_>` and which ones it implements.

`trait InterfaceType` allows describing which traits are required 
when constructing a `DynTrait<_>`,and which ones it implements.

### Construction

To construct a `DynTrait<_>` one can use these associated functions:
    
- from_value:
    Can be constructed from the value directly.
    Requires a value that implements ImplType.
    
- from_ptr:
    Can be constructed from a pointer of a value.
    Requires a value that implements ImplType.
    
- from_any_value:
    Can be constructed from the value directly.Requires a `'static` value.
    
- from_any_ptr
    Can be constructed from a pointer of a value.Requires a `'static` value.

- from_borrowing_value:
    Can be constructed from the value directly.Cannot unerase the DynTrait afterwards.
    
- from_borrowing_ptr
    Can be constructed from a pointer of a value.Cannot unerase the DynTrait afterwards.

DynTrait uses the impls of the value when it's a `&self` or `&mut self` method,
which means that the pointer itself does not have to implement those traits,

### Trait object

`DynTrait<'borrow,Pointer<()>,Interface>` 
can be used as a trait object for any combination of 
the traits listed bellow.

These are the traits:

- Send

- Sync

- Iterator

- DoubleEndedIterator

- std::fmt::Write

- std::io::Write

- std::io::Seek

- std::io::Read

- std::io::BufRead

- Clone 

- Display 

- Debug 

- Default: Can be called as an inherent method.

- Eq 

- PartialEq 

- Ord 

- PartialOrd 

- Hash 

- serde::Deserialize:
    first deserializes from a string,and then calls the objects' Deserialize impl.

- serde::Serialize:
    first calls the objects' Deserialize impl,then serializes that as a string.

### Deconstruction

`DynTrait<_>` can then be unwrapped into a concrete type,
within the same dynamic library/executable that constructed it,
using these (fallible) conversion methods:

- into_unerased:
    Unwraps into a pointer to `T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- as_unerased:
    Unwraps into a `&T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- as_unerased_mut:
    Unwraps into a `&mut T`.
    Where `DynTrait<P<()>,Interface>`'s 
        Interface must equal `<T as ImplType>::Interface`

- into_any_unerased:Unwraps into a pointer to `T`.Requires `T:'static`.

- as_any_unerased:Unwraps into a `&T`.Requires `T:'static`.

- as_any_unerased_mut:Unwraps into a `&mut T`.Requires `T:'static`.


`DynTrait` cannot be converted back if it was created 
using `DynTrait::from_borrowing_*`.

# Passing DynTrait between dynamic libraries

Passing DynTrait between dynamic libraries 
(as in between the dynamic libraries directly loaded by the same binary/dynamic library)
may cause the program to panic at runtime with an error message stating that 
the trait is not implemented for the specific interface.

This can only happen if you are passing DynTrait between dynamic libraries,
or if DynTrait was instantiated in the parent passed to a child,
a DynTrait instantiated in a child dynamic library passed to the parent
should not cause a panic,it would be a bug.

```text
        binary
  _________|___________
lib0      lib1      lib2
  |         |         |
lib00    lib10      lib20
```

In this diagram passing a DynTrait constructed in lib00 to anything other than 
the binary or lib0 will cause the panic to happen if:

- The InterfaceType requires extra traits in the version of the Interface
    that lib1 and lib2 know about (that the binary does not require).

- lib1 or lib2 attempt to call methods that require the traits that were added 
    to the InterfaceType,in versions of that interface that only they know about.






# Examples

### In the Readme

The primary example using `DynTrait<_>` is in the readme.

Readme is in 
[the repository for this crate](https://github.com/rodrimati1992/abi_stable_crates),
[crates.io](https://crates.io/crates/abi_stable),
[lib.rs](https://lib.rs/crates/abi_stable).

### Comparing DynTraits

This is only possible if the erased types don't contain borrows,
and they are not constructed using `DynTrait::from_borrowing_*` methods.

DynTraits wrapping different pointer types can be compared with each other,
it simply uses the values' implementation of PartialEq.

```
use abi_stable::{
    DynTrait,
    erased_types::interfaces::PartialEqInterface,
    std_types::RArc,
};

{
    let left:DynTrait<'static,&(),PartialEqInterface>=
        DynTrait::from_any_ptr(&100,PartialEqInterface);
    
    let mut n100=100;
    let right:DynTrait<'static,&mut (),PartialEqInterface>=
        DynTrait::from_any_ptr(&mut n100,PartialEqInterface);

    assert_eq!(left,right);
}
{
    let left=
        DynTrait::from_any_value(200,PartialEqInterface);

    let right=
        DynTrait::from_any_ptr(RArc::new(200),PartialEqInterface);

    assert_eq!(left,right);
}

```

### Writing to a DynTrait

This is an example of using the `write!()` macro with DynTrait.

```
use abi_stable::{
    DynTrait,
    erased_types::interfaces::FmtWriteInterface,
};

use std::fmt::Write;

let mut buffer=String::new();

let mut wrapped:DynTrait<'static,&mut (),FmtWriteInterface>=
    DynTrait::from_any_ptr(&mut buffer,FmtWriteInterface);

write!(wrapped,"Foo").unwrap();
write!(wrapped,"Bar").unwrap();
write!(wrapped,"Baz").unwrap();

drop(wrapped);

assert_eq!(&buffer[..],"FooBarBaz");


```


### Iteration

Using `DynTrait` as an `Iterator` and `DoubleEndedIterator`.

```
use abi_stable::{
    DynTrait,
    erased_types::interfaces::DEIteratorInterface,
};

let mut wrapped=DynTrait::from_any_value(0..=10,DEIteratorInterface::NEW);

assert_eq!(
    wrapped.by_ref().take(5).collect::<Vec<_>>(),
    vec![0,1,2,3,4]
);

assert_eq!(
    wrapped.rev().collect::<Vec<_>>(),
    vec![10,9,8,7,6,5]
);


```


# Making pointers compatible with DynTrait

To make pointers compatible with DynTrait,they must imlement the 
`abi_stable::pointer_trait::{GetPointerKind,StableDeref,TransmuteElement}` traits 
as shown in the example.

`GetPointerKind` should generally be implemented with `type Kind=PK_SmartPointer`.
The exception is in the case that it is a `#[repr(transparent)]`
wrapper around a `&` or a `&mut`,
in which case it should implement `GetPointerKind<Kind=PK_Reference>` 
or `GetPointerKind<Kind=PK_MutReference>` respectively.

### Example

This is an example of a newtype wrapping an `RBox<T>`.

```rust 
    
use abi_stable::DynTrait;

fn main(){
    let lines="line0\nline1\nline2";
    let mut iter=NewtypeBox::new(lines.lines());

    // The type annotation here is just to show the type,it's not necessary.
    let mut wrapper:DynTrait<'_,NewtypeBox<()>,IteratorInterface>=
        DynTrait::from_borrowing_ptr(iter,IteratorInterface);

    // You can clone the DynTrait! 
    let clone=wrapper.clone();

    assert_eq!( wrapper.next(), Some("line0") );
    assert_eq!( wrapper.next(), Some("line1") );
    assert_eq!( wrapper.next(), Some("line2") );
    assert_eq!( wrapper.next(), None );

    assert_eq!(
        clone.rev().collect::<Vec<_>>(),
        vec!["line2","line1","line0"],
    )

}


/////////////////////////////////////////

use std::ops::{Deref, DerefMut};

use abi_stable::{
    StableAbi,
    InterfaceType,
    impl_InterfaceType,
    std_types::RBox,
    erased_types::IteratorItem,
    pointer_trait::{
        PK_SmartPointer,GetPointerKind,StableDeref,TransmuteElement
    },
    type_level::bools::True,
};

#[repr(transparent)]
#[derive(Default,Clone,StableAbi)]
pub struct NewtypeBox<T>{
    box_:RBox<T>,
}

impl<T> NewtypeBox<T>{
    pub fn new(value:T)->Self{
        Self{
            box_:RBox::new(value)
        }
    }
}

impl<T> Deref for NewtypeBox<T>{
    type Target=T;

    fn deref(&self)->&T{
        &*self.box_
    }
}

impl<T> DerefMut for NewtypeBox<T>{
    fn deref_mut(&mut self)->&mut T{
        &mut *self.box_
    }
}

unsafe impl<T> GetPointerKind for NewtypeBox<T>{
    type Kind=PK_SmartPointer;
}

unsafe impl<T> StableDeref for NewtypeBox<T> {}

unsafe impl<T,O> TransmuteElement<O> for NewtypeBox<T>
where 
    // Using this to ensure that the pointer is safe to wrap,
    // while this is not necessary for `RBox<T>`,
    // it might be for some other pointer type.
    RBox<T>:TransmuteElement<O,Kind=Self::Kind>
{
    type TransmutedPtr = NewtypeBox<O>;
}

/////////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
pub struct IteratorInterface;

impl_InterfaceType!{
    impl InterfaceType for IteratorInterface {
        type Iterator = True;
        type DoubleEndedIterator = True;
        type Clone = True;
        type Debug = True;
    }
}

impl<'a> IteratorItem<'a> for IteratorInterface{
    type Item=&'a str;
}

```

    
    */
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(
        inside_abi_stable_crate,
        prefix_bound="I:InterfaceBound<'borr>",
        bound="<I as SharedStableAbi>::StaticEquivalent:InterfaceBound<'static>",
        bound="VTable<'borr,P,I>:SharedStableAbi",
        tag="<I as InterfaceBound<'borr>>::TAG",
    )]
    pub struct DynTrait<'borr,P,I> 
    where I:InterfaceBound<'borr>
    {
        pub(super) object: ManuallyDrop<P>,
        vtable: *const VTable<'borr,P,I>,
        _marker:PhantomData<extern fn()->Tuple2<I,RStr<'borr>>>,
        _marker2:UnsafeIgnoredType<Rc<()>>,

    }

    impl DynTrait<'static,&'static (),()> {
        /// Constructs the `DynTrait<_>` from a `T:ImplType`.
        ///
        /// Use this whenever possible instead of `from_any_value`,
        /// because it produces better error messages when unerasing the `DynTrait<_>`
        pub fn from_value<T>(object: T) -> DynTrait<'static,RBox<()>,T::Interface>
        where
            T: ImplType,
            T::Interface:InterfaceBound<'static>,
            T: GetVtable<'static,T,RBox<()>,RBox<T>,<T as ImplType>::Interface>,
        {
            let object = RBox::new(object);
            DynTrait::from_ptr(object)
        }

        /// Constructs the `DynTrait<_>` from a pointer to a `T:ImplType`.
        ///
        /// Use this whenever possible instead of `from_any_ptr`,
        /// because it produces better error messages when unerasing the `DynTrait<_>`
        pub fn from_ptr<P, T>(object: P) -> DynTrait<'static,P::TransmutedPtr,T::Interface>
        where
            T: ImplType,
            T::Interface:InterfaceBound<'static>,
            T: GetVtable<'static,T,P::TransmutedPtr,P,<T as ImplType>::Interface>,
            P: StableDeref<Target = T>+TransmuteElement<()>,
        {
            DynTrait {
                object: unsafe{
                    // The lifetime here is 'static,so it's fine to erase the type.
                    ManuallyDrop::new(object.transmute_element(<()>::T))
                },
                vtable: T::get_vtable(),
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }

        /// Constructs the `DynTrait<_>` from a type that doesn't borrow anything.
        pub fn from_any_value<T,I>(object: T,interface:I) -> DynTrait<'static,RBox<()>,I>
        where
            T:'static,
            I:InterfaceBound<'static>,
            InterfaceFor<T,I,True> : GetVtable<'static,T,RBox<()>,RBox<T>,I>,
        {
            let object = RBox::new(object);
            DynTrait::from_any_ptr(object,interface)
        }

        /// Constructs the `DynTrait<_>` from a pointer to a 
        /// type that doesn't borrow anything.
        pub fn from_any_ptr<P, T,I>(
            object: P,
            _interface:I
        ) -> DynTrait<'static,P::TransmutedPtr,I>
        where
            I:InterfaceBound<'static>,
            T:'static,
            InterfaceFor<T,I,True>: GetVtable<'static,T,P::TransmutedPtr,P,I>,
            P: StableDeref<Target = T>+TransmuteElement<()>,
        {
            DynTrait {
                object: unsafe{
                    // The lifetime here is 'static,so it's fine to erase the type.
                    ManuallyDrop::new(object.transmute_element(<()>::T))
                },
                vtable: <InterfaceFor<T,I,True>>::get_vtable(),
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
        
        /// Constructs the `DynTrait<_>` from a value with a `'borr` borrow.
        ///
        /// Cannot unerase the DynTrait afterwards.
        pub fn from_borrowing_value<'borr,T,I>(
            object: T,
            interface:I,
        ) -> DynTrait<'borr,RBox<()>,I>
        where
            T:'borr,
            I:InterfaceBound<'borr>,
            InterfaceFor<T,I,False> : GetVtable<'borr,T,RBox<()>,RBox<T>,I>,
        {
            let object = RBox::new(object);
            DynTrait::from_borrowing_ptr(object,interface)
        }

        /// Constructs the `DynTrait<_>` from a pointer to the erased type
        /// with a `'borr` borrow.
        ///
        /// Cannot unerase the DynTrait afterwards.
        pub fn from_borrowing_ptr<'borr,P, T,I>(
            object: P,
            _interface:I
        ) -> DynTrait<'borr,P::TransmutedPtr,I>
        where
            T:'borr,
            I:InterfaceBound<'borr>,
            InterfaceFor<T,I,False>: GetVtable<'borr,T,P::TransmutedPtr,P,I>,
            P: StableDeref<Target = T>+TransmuteElement<()>,
        {
            DynTrait {
                object: unsafe{
                    // The lifetime here is 'static,so it's fine to erase the type.
                    ManuallyDrop::new(object.transmute_element(<()>::T))
                },
                vtable: <InterfaceFor<T,I,False>>::get_vtable(),
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }


    impl<P,I> DynTrait<'static,P,I> 
    where 
        I: InterfaceBound<'static>
    {
        /// Allows checking whether 2 `DynTrait<_>`s have a value of the same type.
        ///
        /// Notes:
        ///
        /// - Types from different dynamic libraries/executables are 
        /// never considered equal.
        ///
        /// - `DynTrait`s constructed using `DynTrait::from_borrowing_*`
        /// are never considered to wrap the same type.
        pub fn is_same_type<Other,I2>(&self,other:&DynTrait<'static,Other,I2>)->bool
        where I2:InterfaceBound<'static>
        {
            self.vtable_address()==other.vtable_address()||
            self.vtable().type_info().is_compatible(other.vtable().type_info())
        }
    }

    impl<'borr,P,I> DynTrait<'borr,P,I> 
    where 
        I: InterfaceBound<'borr>
    {
        pub(super) fn vtable<'a>(&self) -> &'a VTable<'borr,P,I>{
            unsafe {
                &*(((self.vtable as usize)&PTR_MASK) as *const VTable<'borr,P,I>)
            }
        }

        pub(super)fn vtable_address(&self) -> usize {
            (self.vtable as usize)&PTR_MASK
        }

        pub(super)fn vtable_ptr_flags(&self) -> usize {
            (self.vtable as usize)&PTR_FLAGS
        }

        pub(super) fn as_abi(&self) -> &ErasedObject
        where
            P: Deref,
        {
            self.object()
        }

        #[allow(dead_code)]
        pub(super) fn as_abi_mut(&mut self) -> &mut ErasedObject
        where
            P: DerefMut,
        {
            self.object_mut()
        }

        /// Returns the address of the wrapped object.
        ///
        /// This will not change between calls for the same `DynTrait<_>`.
        pub fn object_address(&self) -> usize
        where
            P: Deref,
        {
            self.object() as *const ErasedObject as usize
        }

        pub(super) fn object(&self) -> &ErasedObject
        where
            P: Deref,
        {
            unsafe { self.object_as() }
        }
        pub(super) fn object_mut(&mut self) -> &mut ErasedObject
        where
            P: DerefMut,
        {
            unsafe { self.object_as_mut() }
        }

        unsafe fn object_as<T>(&self) -> &T
        where
            P: Deref,
        {
            &*((&**self.object) as *const P::Target as *const T)
        }
        unsafe fn object_as_mut<T>(&mut self) -> &mut T
        where
            P: DerefMut,
        {
            &mut *((&mut **self.object) as *mut P::Target as *mut T)
        }
    }

    impl<'borr,P,I> DynTrait<'borr,P,I> 
    where 
        I: InterfaceBound<'borr>
    {
        /// The uid in the vtable has to be the same as the one for T,
        /// otherwise it was not created from that T in the library that declared the opaque type.
        pub(super) fn check_same_destructor_opaque<A,T>(&self) -> Result<(), UneraseError>
        where
            P: TransmuteElement<T>,
            A: GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            let t_vtable:&VTable<'borr,P,I> = A::get_vtable();
            if self.vtable_address() == t_vtable as *const _ as usize
                || self.vtable().type_info().is_compatible(t_vtable.type_info())
            {
                Ok(())
            } else {
                Err(UneraseError {
                    expected_vtable_address: t_vtable as *const _ as usize,
                    expected_type_info:t_vtable.type_info(),
                    found_vtable_address: self.vtable as usize,
                    found_type_info:self.vtable().type_info(),
                })
            }
        }

        /// Unwraps the `DynTrait<_>` into a pointer of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn into_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError>
        where
            P: TransmuteElement<T>,
            P::Target:Sized,
            T: ImplType + GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { 
                let this=ManuallyDrop::new(self);
                Ok(ptr::read(&*this.object).transmute_element(T::T)) 
            }
        }

        /// Unwraps the `DynTrait<_>` into a reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_unerased<T>(&self) -> Result<&T, UneraseError>
        where
            P: Deref + TransmuteElement<T>,
            T: ImplType + GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { Ok(self.object_as()) }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to implement ImplType.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError>
        where
            P: DerefMut + TransmuteElement<T>,
            T: ImplType + GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<T,T>()?;
            unsafe { Ok(self.object_as_mut()) }
        }


        /// Unwraps the `DynTrait<_>` into a pointer of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn into_any_unerased<T>(self) -> Result<P::TransmutedPtr, UneraseError>
        where
            T:'static,
            P: TransmuteElement<T>,
            P::Target:Sized,
            Self:DynTraitBound<'borr>,
            InterfaceFor<T,I,True>: GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,I,True>,T>()?;
            unsafe {
                unsafe { 
                    let this=ManuallyDrop::new(self);
                    Ok(ptr::read(&*this.object).transmute_element(T::T)) 
                }
            }
        }

        /// Unwraps the `DynTrait<_>` into a reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_any_unerased<T>(&self) -> Result<&T, UneraseError>
        where
            T:'static,
            P: Deref + TransmuteElement<T>,
            Self:DynTraitBound<'borr>,
            InterfaceFor<T,I,True>: GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,I,True>,T>()?;
            unsafe { Ok(self.object_as()) }
        }

        /// Unwraps the `DynTrait<_>` into a mutable reference of 
        /// the concrete type that it was constructed with.
        ///
        /// T is required to not borrow anything.
        ///
        /// # Errors
        ///
        /// This will return an error in any of these conditions:
        ///
        /// - It is called in a dynamic library/binary outside
        /// the one from which this `DynTrait<_>` was constructed.
        ///
        /// - The DynTrait was constructed using a `from_borrowing_*` method
        ///
        /// - `T` is not the concrete type this `DynTrait<_>` was constructed with.
        ///
        pub fn as_any_unerased_mut<T>(&mut self) -> Result<&mut T, UneraseError>
        where
            P: DerefMut + TransmuteElement<T>,
            Self:DynTraitBound<'borr>,
            InterfaceFor<T,I,True>: GetVtable<'borr,T,P,P::TransmutedPtr,I>,
        {
            self.check_same_destructor_opaque::<InterfaceFor<T,I,True>,T>()?;
            unsafe { Ok(self.object_as_mut()) }
        }

    }


    mod private_struct {
        pub struct PrivStruct;
    }
    use self::private_struct::PrivStruct;

    
    /// This is used to make sure that reborrowing does not change 
    /// the Send-ness or Sync-ness of the pointer.
    pub trait ReborrowBounds<SendNess,SyncNess>{}

    // If it's reborrowing,it must have either both Sync+Send or neither.
    impl ReborrowBounds<False,False> for PrivStruct {}
    impl ReborrowBounds<True ,True > for PrivStruct {}

    
    impl<'borr,P,I> DynTrait<'borr,P,I> 
    where 
        I:InterfaceBound<'borr>
    {
        /// Creates a shared reborrow of this DynTrait.
        ///
        /// The reborrowed DynTrait cannot use these methods:
        /// 
        /// - DynTrait::default
        /// 
        pub fn reborrow<'re>(&'re self)->DynTrait<'borr,&'re (),I> 
        where
            P:Deref<Target=()>,
            PrivStruct:ReborrowBounds<I::Send,I::Sync>,
        {
            // Reborrowing will break if I add extra functions that operate on `P`.
            DynTrait {
                object: ManuallyDrop::new(&**self.object),
                vtable: ((self.vtable as usize) | PTR_FLAG_IS_BORROWED)as *const _,
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }

        /// Creates a mutable reborrow of this DynTrait.
        ///
        /// The reborrowed DynTrait cannot use these methods:
        /// 
        /// - DynTrait::default
        /// 
        /// - DynTrait::clone
        /// 
        pub fn reborrow_mut<'re>(&'re mut self)->DynTrait<'borr,&'re mut (),I> 
        where
            P:DerefMut<Target=()>,
            PrivStruct:ReborrowBounds<I::Send,I::Sync>,
        {
            // Reborrowing will break if I add extra functions that operate on `P`.
            DynTrait {
                object: ManuallyDrop::new(&mut **self.object),
                vtable: ((self.vtable as usize) | PTR_FLAG_IS_BORROWED)as *const _,
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }


    impl<'borr,P,I> DynTrait<'borr,P,I> 
    where 
        I:InterfaceBound<'borr>+'borr
    {
        /// Constructs a DynTrait<P,I> with a `P`,using the same vtable.
        /// `P` must come from a function in the vtable,
        /// or come from a copy of `P:Copy+GetPointerKind<Kind=PK_Reference>`,
        /// to ensure that it is compatible with the functions in it.
        pub(super) fn from_new_ptr(&self, object: P) -> Self {
            Self {
                object:ManuallyDrop::new(object),
                vtable: self.vtable,
                _marker:PhantomData,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }

/**
Constructs a `DynTrait<P,I>` with the default value for `P`.

# Reborrowing

This cannot be called with a reborrowed DynTrait:

```compile_fail
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::DefaultInterface,
# };
let object=DynTrait::from_any_value((),DefaultInterface);
let borrow=object.reborrow();
let _=borrow.default();

```

```compile_fail
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::DefaultInterface,
# };
let object=DynTrait::from_any_value((),DefaultInterface);
let borrow=object.reborrow_mut();
let _=borrow.default();

```
 */
        pub fn default(&self) -> Self
        where
            P: Deref + GetPointerKind<Kind=PK_SmartPointer>,
            I: InterfaceType<Default = True>,
        {
            let new = self.vtable().default_ptr()();
            self.from_new_ptr(new)
        }

        /// It serializes a `DynTrait<_>` into a string by using 
        /// `<ConcreteType as SerializeImplType>::serialize_impl`.
        pub fn serialized<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>
        where
            P: Deref,
            I: InterfaceType<Serialize = True>,
        {
            self.vtable().serialize()(self.as_abi()).into_result()
        }

        /// Deserializes a string into a `DynTrait<_>`,by using 
        /// `<I as DeserializeOwnedInterface>::deserialize_impl`.
        pub fn deserialize_owned_from_str(s: &str) -> Result<Self, RBoxError>
        where
            P: 'borr+Deref,
            I: DeserializeOwnedInterface<'borr,Deserialize = True, Deserialized = Self>,
        {
            s.piped(RStr::from).piped(I::deserialize_impl)
        }

        /// Deserializes a `&'borr str` into a `DynTrait<'borr,_>`,by using 
        /// `<I as DeserializeBorrowedInterface<'borr>>::deserialize_impl`.
        pub fn deserialize_borrowing_from_str(s: &'borr str) -> Result<Self, RBoxError>
        where
            P: 'borr+Deref,
            I: DeserializeBorrowedInterface<'borr,Deserialize = True, Deserialized = Self>,
        {
            s.piped(RStr::from).piped(I::deserialize_impl)
        }
    }

    impl<'borr,P,I> Drop for DynTrait<'borr,P,I>
    where I:InterfaceBound<'borr>
    {
        fn drop(&mut self){
            unsafe{
                let vtable=self.vtable();

                if (self.vtable_ptr_flags()&PTR_FLAG_IS_BORROWED)==PTR_FLAG_IS_BORROWED {
                    // Do nothing
                }else{
                    vtable.drop_ptr()(&mut *self.object);
                }
            }
        }
    }

}


const PTR_FLAGS:usize=0b1111;
const PTR_MASK:usize=!PTR_FLAGS;
const PTR_FLAG_IS_BORROWED:usize=0b_0001;


pub use self::priv_::DynTrait;

//////////////////////



mod clone_impl{
    pub trait CloneImpl<PtrKind>{
        fn clone_impl(&self) -> Self;
    }
}
use self::clone_impl::CloneImpl;


/// This impl is for smart pointers.
impl<'borr,P, I> CloneImpl<PK_SmartPointer> for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr,Clone = True>+'borr,
{
    fn clone_impl(&self) -> Self {
        let vtable = self.vtable();
        let new = vtable.clone_ptr()(&*self.object);
        self.from_new_ptr(new)
    }
}

/// This impl is for references.
impl<'borr,P, I> CloneImpl<PK_Reference> for DynTrait<'borr,P,I>
where
    P: Deref+Copy,
    I: InterfaceBound<'borr,Clone = True>+'borr,
{
    fn clone_impl(&self) -> Self {
        self.from_new_ptr(*self.object)
    }
}


/**
Clone is implemented for references and smart pointers,
using `GetPointerKind` to decide whether `P` is a smart pointer or a reference.

DynTrait does not implement Clone if P==`&mut ()` :

```compile_fail
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::CloneInterface,
# };

let object=DynTrait::from_any_value((),());
let borrow=object.reborrow_mut();
let _=borrow.clone();

```

*/
impl<'borr,P, I> Clone for DynTrait<'borr,P,I>
where
    P: Deref+GetPointerKind,
    I: InterfaceBound<'borr>,
    Self:CloneImpl<<P as GetPointerKind>::Kind>,
{
    fn clone(&self) -> Self {
        self.clone_impl()
    }
}

//////////////////////


impl<'borr,P, I> Display for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr,Display = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().display(), f)
    }
}

impl<'borr,P, I> Debug for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr,Debug = True>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        adapt_std_fmt::<ErasedObject>(self.object(), self.vtable().debug(), f)
    }
}

/**
First it serializes a `DynTrait<_>` into a string by using 
<ConcreteType as SerializeImplType>::serialize_impl,
then it serializes the string.

*/
/// ,then it .
impl<'borr,P, I> Serialize for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr,Serialize = True>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.vtable().serialize()(self.as_abi())
            .into_result()
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}

/// First it Deserializes a string,then it deserializes into a 
/// `DynTrait<_>`,by using `<I as DeserializeOwnedInterface>::deserialize_impl`.
impl<'de,'borr:'de, P, I> Deserialize<'de> for DynTrait<'borr,P,I>
where
    P: Deref+'borr,
    I: InterfaceBound<'borr>+'borr,
    I: DeserializeOwnedInterface<'borr,Deserialize = True, Deserialized = Self>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        I::deserialize_impl(RStr::from(&*s)).map_err(de::Error::custom)
    }
}

impl<P, I> Eq for DynTrait<'static,P,I>
where
    Self: PartialEq,
    P: Deref,
    I: InterfaceBound<'static,Eq = True>,
{
}

impl<P, P2, I> PartialEq<DynTrait<'static,P2,I>> for DynTrait<'static,P,I>
where
    P: Deref,
    P2: Deref,
    I: InterfaceBound<'static,PartialEq = True>,
{
    fn eq(&self, other: &DynTrait<'static,P2,I>) -> bool {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return false;
        }

        self.vtable().partial_eq()(self.as_abi(), other.as_abi())
    }
}

impl<P, I> Ord for DynTrait<'static,P,I>
where
    P: Deref,
    I: InterfaceBound<'static,Ord = True>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return self.vtable_address().cmp(&other.vtable_address());
        }

        self.vtable().cmp()(self.as_abi(), other.as_abi()).into()
    }
}

impl<P, P2, I> PartialOrd<DynTrait<'static,P2,I>> for DynTrait<'static,P,I>
where
    P: Deref,
    P2: Deref,
    I: InterfaceBound<'static,PartialOrd = True>,
    Self: PartialEq<DynTrait<'static,P2,I>>,
{
    fn partial_cmp(&self, other: &DynTrait<'static,P2,I>) -> Option<Ordering> {
        // unsafe: must check that the vtable is the same,otherwise return a sensible value.
        if !self.is_same_type(other) {
            return Some(self.vtable_address().cmp(&other.vtable_address()));
        }

        self.vtable().partial_cmp()(self.as_abi(), other.as_abi())
            .map(IntoReprRust::into_rust)
            .into()
    }
}

impl<'borr,P, I> Hash for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr,Hash = True>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.vtable().hash()(self.as_abi(), HasherObject::new(state))
    }
}


//////////////////////////////////////////////////////////////////


impl<'borr,P, I,Item> Iterator for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,Iterator = True,IteratorItem=Item>,
{
    type Item=Item;

    fn next(&mut self)->Option<Item>{
        let vtable=self.vtable();
        (vtable.iter().next)(self.as_abi_mut()).into_rust()
    }

    fn nth(&mut self,nth:usize)->Option<Item>{
        let vtable=self.vtable();
        (vtable.iter().nth)(self.as_abi_mut(),nth).into_rust()
    }

    fn size_hint(&self)->(usize,Option<usize>){
        let vtable=self.vtable();
        let tuple=(vtable.iter().size_hint)(self.as_abi()).into_rust();
        (tuple.0,tuple.1.into_rust())
    }

    fn count(mut self)->usize{
        let vtable=self.vtable();
        (vtable.iter().count)(self.as_abi_mut())
    }

    fn last(mut self)->Option<Item>{
        let vtable=self.vtable();
        (vtable.iter().last)(self.as_abi_mut()).into_rust()
    }
}


impl<'borr,P, I,Item> DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,Iterator = True,IteratorItem=Item>,
{
/**
Eagerly skips n elements from the iterator.

This method is faster than using `Iterator::skip`.

# Example

```
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::IteratorInterface,
#     std_types::RVec,
#     traits::IntoReprC,
# };

let mut iter=0..20;
let mut wrapped=DynTrait::from_any_ptr(&mut iter,IteratorInterface::NEW);

assert_eq!(wrapped.next(),Some(0));

wrapped.skip_eager(2);

assert_eq!(wrapped.next(),Some(3));
assert_eq!(wrapped.next(),Some(4));
assert_eq!(wrapped.next(),Some(5));

wrapped.skip_eager(2);

assert_eq!(wrapped.next(),Some(8));
assert_eq!(wrapped.next(),Some(9));

wrapped.skip_eager(9);

assert_eq!(wrapped.next(),Some(19));
assert_eq!(wrapped.next(),None    );



```


*/
    pub fn skip_eager(&mut self, n: usize){
        let vtable=self.vtable();
        (vtable.iter().skip_eager)(self.as_abi_mut(),n);
    }


/**
Extends the `RVec<Item>` with the `self` Iterator.

Extends `buffer` with as many elements of the iterator as `taking` specifies:

- RNone: Yields all elements.Use this with care,since Iterators can be infinite.

- RSome(n): Yields n elements.

### Example

```
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::IteratorInterface,
#     std_types::{RVec,RSome},
#     traits::IntoReprC,
# };

let mut wrapped=DynTrait::from_any_value(0.. ,IteratorInterface::NEW);

let mut buffer=vec![ 101,102,103 ].into_c();
wrapped.extending_rvec(&mut buffer,RSome(5));
assert_eq!(
    &buffer[..],
    &*vec![101,102,103,0,1,2,3,4]
);

assert_eq!( wrapped.next(),Some(5));
assert_eq!( wrapped.next(),Some(6));
assert_eq!( wrapped.next(),Some(7));

```
*/
    pub fn extending_rvec(&mut self,buffer:&mut RVec<Item>,taking:ROption<usize>){
        let vtable=self.vtable();
        (vtable.iter().extending_rvec)(self.as_abi_mut(),buffer,taking);
    }
}


//////////////////////////////////////////////////////////////////


impl<'borr,P, I,Item> DoubleEndedIterator for DynTrait<'borr,P,I>
where
    Self:Iterator<Item=Item>,
    P: DerefMut,
    I: InterfaceBound<'borr,DoubleEndedIterator = True,IteratorItem=Item>,
{

    fn next_back(&mut self)->Option<Item>{
        let vtable=self.vtable();
        (vtable.back_iter().next_back)(self.as_abi_mut()).into_rust()
    }
}


impl<'borr,P, I,Item> DynTrait<'borr,P,I>
where
    Self:Iterator<Item=Item>,
    P: DerefMut,
    I: InterfaceBound<'borr,DoubleEndedIterator = True,IteratorItem=Item>,
{
    pub fn nth_back_(&mut self,nth:usize)->Option<Item>{
        let vtable=self.vtable();
        (vtable.back_iter().nth_back)(self.as_abi_mut(),nth).into_rust()
    }

/**
Extends the `RVec<Item>` with the back of the `self` DoubleEndedIterator.

Extends `buffer` with as many elements of the iterator as `taking` specifies:

- RNone: Yields all elements.Use this with care,since Iterators can be infinite.

- RSome(n): Yields n elements.

### Example

```
# use abi_stable::{
#     DynTrait,
#     erased_types::interfaces::DEIteratorInterface,
#     std_types::{RVec,RNone},
#     traits::IntoReprC,
# };

let mut wrapped=DynTrait::from_any_value(0..=3 ,DEIteratorInterface::NEW);

let mut buffer=vec![ 101,102,103 ].into_c();
wrapped.extending_rvec_back(&mut buffer,RNone);
assert_eq!(
    &buffer[..],
    &*vec![101,102,103,3,2,1,0]
)

```

*/
    pub fn extending_rvec_back(&mut self,buffer:&mut RVec<Item>,taking:ROption<usize>){
        let vtable=self.vtable();
        (vtable.back_iter().extending_rvec_back)(self.as_abi_mut(),buffer,taking);
    }
}


//////////////////////////////////////////////////////////////////


impl<'borr,P,I> fmtWrite for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,FmtWrite = True>,
{
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error>{
        let vtable = self.vtable();
        match vtable.fmt_write_str()(self.as_abi_mut(),s.into()) {
            ROk(_)=>Ok(()),
            RErr(_)=>Err(fmt::Error),
        }
    }
}



//////////////////////////////////////////////////////////////////


#[inline]
fn to_io_result<T,U>(res:RResult<T,RIoError>)->io::Result<U>
where
    T:Into<U>
{
    match res {
        ROk(v)=>Ok(v.into()),
        RErr(e)=>Err(e.into()),
    }
}


/////////////


impl<'borr,P,I> io::Write for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,IoWrite = True>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize>{
        let vtable = self.vtable().io_write();

        to_io_result((vtable.write)(self.as_abi_mut(),buf.into()))
    }
    fn flush(&mut self) -> io::Result<()>{
        let vtable = self.vtable().io_write();

        to_io_result((vtable.flush)(self.as_abi_mut()))
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        let vtable = self.vtable().io_write();

        to_io_result((vtable.write_all)(self.as_abi_mut(),buf.into()))
    }
}


/////////////


impl<'borr,P,I> io::Read for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,IoRead = True>,
{
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>{
        let vtable = self.vtable().io_read();

        to_io_result((vtable.read)(self.as_abi_mut(),buf.into()))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        let vtable = self.vtable().io_read();

        to_io_result((vtable.read_exact)(self.as_abi_mut(),buf.into()))
    }

}


/////////////


impl<'borr,P,I> io::BufRead for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,IoRead = True,IoBufRead = True>,
{
    fn fill_buf(&mut self) -> io::Result<&[u8]>{
        let vtable = self.vtable().io_bufread();

        to_io_result((vtable.fill_buf)(self.as_abi_mut()))
    }

    fn consume(&mut self, ammount:usize ){
        let vtable = self.vtable().io_bufread();

        (vtable.consume)(self.as_abi_mut(),ammount)
    }

}

/////////////


impl<'borr,P,I> io::Seek for DynTrait<'borr,P,I>
where
    P: DerefMut,
    I: InterfaceBound<'borr,IoSeek = True>,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64>{
        let vtable = self.vtable();

        to_io_result(vtable.io_seek()(self.as_abi_mut(),pos.into()))
    }
}


//////////////////////////////////////////////////////////////////

unsafe impl<'borr,P,I> Send for DynTrait<'borr,P,I>
where
    P: Send,
    I: InterfaceBound<'borr,Send = True>,
{}


unsafe impl<'borr,P,I> Sync for DynTrait<'borr,P,I>
where
    P: Sync,
    I: InterfaceBound<'borr,Sync = True>,
{}


//////////////////////////////////////////////////////////////////

mod sealed {
    use super::*;
    pub trait Sealed {}
    impl<'borr,P,I> Sealed for DynTrait<'borr,P,I> 
    where I:InterfaceBound<'borr>
    {}
}
use self::sealed::Sealed;

/// For accessing the Interface of a `DynTrait<Pointer<ZeroSized< Interface >>>`.
pub trait DynTraitBound<'borr>: Sealed {
    type Interface: InterfaceType;
}

impl<'borr,P, I> DynTraitBound<'borr> for DynTrait<'borr,P,I>
where
    P: Deref,
    I: InterfaceBound<'borr>,
{
    type Interface = I;
}


/// For accessing the `Interface` in a `DynTrait<Pointer<ZeroSized< Interface >>>`.
pub type GetVWInterface<'borr,This>=
    <This as DynTraitBound<'borr>>::Interface;


//////////////////////////////////////////////////////////////////

/// Error for `DynTrait<_>` being unerased into the wrong type
/// with one of the `*unerased*` methods.
#[derive(Debug,Copy, Clone)]
pub struct UneraseError {
    expected_vtable_address: usize,
    expected_type_info:&'static TypeInfo,
    found_vtable_address: usize,
    found_type_info:&'static TypeInfo,
}


impl fmt::Display for UneraseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl ::std::error::Error for UneraseError {}

//////////////////////////////////////////////////////////////////
