use std::{
    cmp::{Ordering,PartialEq,Eq,Ord,PartialOrd},
    fmt::{self,Debug,Display},
    hash::{Hash,Hasher},
    marker::PhantomData,
    mem::{self,ManuallyDrop},
    ops::Deref,
};

use crate::{
    erased_types::{
        c_functions,
        trait_objects::{
            HasherObject,
        },
    },
    marker_type::{ErasedObject,UnsafeIgnoredType},
    nonexhaustive_enum::{
        GetVTable,NonExhaustiveVtable,InterfaceBound,GetEnumInfo,GetNonExhaustive,
        ValidDiscriminant,EnumInfo,
        SerializeEnum,DeserializeOwned,DeserializeBorrowed,
    },
    pointer_trait::TransmuteElement,
    type_level::{
        impl_enum::Implemented,
        trait_marker,
    },
    sabi_types::{StaticRef},
    std_types::{RStr,Tuple2,RBoxError,RCow},
    traits::IntoReprRust,
};

use core_extensions::{
    SelfOps,
    utils::transmute_ignore_size,
};

use serde::{ser,de,Serialize,Deserialize,Serializer,Deserializer};



#[cfg(test)]
mod tests;



/**

A generic type for all ffi-safe non-exhaustive enums.

This type allows adding variants to enums it wraps in ABI compatible versions of a library.

# Generic parameters

<h3> `E` </h3>

This is the enums that this was constructed from,
and can be unwrapped back into if it's one of the valid variants in this context.

<h3> `S` </h3>

The storage type,used to store the enum opaquely.

This has to be at least the size and alignment of the wrapped enum.

This is necessary because:

- The compiler assumes that an enum cannot be a variant outside the ones it sees.

- To give some flexibility to grow the enum in semver compatible versions of a library.

<h3> `I` </h3>

The interface of the enum(it implements `InterfaceType`),
determining which traits are required when constructing `NonExhaustive<>`
and which are available afterwards.

### Example

Say that we define an error type for a library.


Version 1.0.
```
use abi_stable::{
    StableAbi,
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    std_types::RString,
    sabi_trait,
};

#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;8]",
    traits(Debug,Clone,PartialEq),
)))]
pub enum Error{
    #[doc(hidden)]
    __NonExhaustive,
    CouldNotFindItem{
        name:RString,
    },
    OutOfStock{
        id:usize,
        name:RString,
    },
}


fn returns_could_not_find_item(name:RString)->NonExhaustiveFor<Error>{
    let e=Error::CouldNotFindItem{name};
    NonExhaustive::new(e)
}

fn returns_out_of_stock(id:usize,name:RString)->NonExhaustiveFor<Error>{
    let e=Error::OutOfStock{id,name};
    NonExhaustive::new(e)
}

```

Then in 1.1 we add another error variant,returned only by new library functions.

```
use abi_stable::{
    StableAbi,
    nonexhaustive_enum::{NonExhaustiveFor,NonExhaustive},
    std_types::RString,
    sabi_trait,
};

#[repr(u8)]
#[derive(StableAbi,Debug,Clone,PartialEq)]
#[sabi(kind(WithNonExhaustive(
    size="[usize;8]",
    traits(Debug,Clone,PartialEq),
)))]
pub enum Error{
    #[doc(hidden)]
    __NonExhaustive,
    CouldNotFindItem{
        name:RString,
    },
    OutOfStock{
        id:usize,
        name:RString,
    },
    InvalidItemId{
        id:usize,
    },
}


fn returns_invalid_item_id()->NonExhaustiveFor<Error>{
    NonExhaustive::new(Error::InvalidItemId{id:100})
}

```

If a library user attempted to unwrap `Error::InvalidItemId` 
(using NonExhaustive::as_enum/as_enum_mut/into_enum)
with the 1.0 version of `Error` they would get an `Err(..)` back.


*/
#[repr(C)]
#[derive(StableAbi)]
#[sabi(
    unconstrained(E,I),
    bound="E: GetNonExhaustive<S>",
    bound="I: InterfaceBound",
    tag="<I as InterfaceBound>::TAG",
    phantom_field="non_exhaustive:<E as GetNonExhaustive<S>>::NonExhaustive",
)]
pub struct NonExhaustive<E,S,I>{
    fill:ManuallyDrop<S>,
    vtable:StaticRef<NonExhaustiveVtable<E,S,I>>,
    _marker:PhantomData<Tuple2<UnsafeIgnoredType<E>,UnsafeIgnoredType<I>>>,
}

/// The type of a `NonExhaustive<>` wrapping the enum E,
/// using the `E`'s  default storage and interface.
pub type NonExhaustiveFor<E>=
    NonExhaustive<
        E,
        <E as GetEnumInfo>::DefaultStorage,
        <E as GetEnumInfo>::DefaultInterface,
    >;

/// The type of a `NonExhaustive<>` wrapping the enum E,
/// using the `E`'s  default storage and a custom interface.
pub type NonExhaustiveWI<E,I>=
    NonExhaustive<
        E,
        <E as GetEnumInfo>::DefaultStorage,
        I,
    >;

/// The type of a `NonExhaustive<>` wrapping the enum E,
/// using a custom storage and the `E`'s default interface.
pub type NonExhaustiveWS<E,S>=
    NonExhaustive<
        E,
        S,
        <E as GetEnumInfo>::DefaultInterface,
    >;



impl<E,S,I> NonExhaustive<E,S,I>{

/**
Constructs a `NonExhaustive<>` from `value` using its default interface and storage.

# Panic

This panics if the storage has an alignment or size smaller than that of `E`.
*/
    #[inline]
    pub fn new(value:E)->Self
    where
        E:GetVTable<S,I,DefaultStorage=S,DefaultInterface=I>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

/*
Constructs a `NonExhaustive<>` from `value` using its default storage 
and a custom interface.

# Panic

This panics if the storage has an alignment or size smaller than that of `E`.
*/
    #[inline]
    pub fn with_interface(value:E)->Self
    where
        E:GetVTable<S,I,DefaultStorage=S>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

/**    
Constructs a `NonExhaustive<>` from `value` using its default interface
and a custom storage.

# Panic

This panics if the storage has an alignment or size smaller than that of `E`.
*/
    #[inline]
    pub fn with_storage(value:E)->Self
    where
        E:GetVTable<S,I,DefaultInterface=I>,
    {
        NonExhaustive::with_storage_and_interface(value)
    }

/**    
Constructs a `NonExhaustive<>` from `value` using both a custom interface and storage.

# Panic

This panics if the storage has an alignment or size smaller than that of `E`.
*/
    pub fn with_storage_and_interface(value:E)->Self
    where 
        E:GetVTable<S,I>,
    {
        unsafe{
            NonExhaustive::with_vtable(value,E::VTABLE_REF)
        }
    }
    pub(super) unsafe fn with_vtable(
        value:E,
        vtable:StaticRef<NonExhaustiveVtable<E,S,I>>
    )->Self{
        Self::assert_fits_within_storage();

        let mut this=Self{
            fill:mem::uninitialized(),
            vtable,
            _marker:PhantomData
        };
        
        (&mut this.fill as *mut ManuallyDrop<S> as *mut E).write(value);

        this
    }


    /// Checks that the alignment of `E` is correct,returning `true` if it is.
    pub fn check_alignment()->bool{
        let align_enum=std::mem::align_of::<E>();
        let align_storage=std::mem::align_of::<S>();
        align_enum <= align_storage
    }

    /// Checks that the size of `E` is correct,returning `true` if it is.
    pub fn check_size()->bool{
        let size_enum=std::mem::size_of::<E>();
        let size_storage=std::mem::size_of::<S>();
        size_enum <= size_storage
    }

    /// Asserts that `E` fits within `S`,with the correct alignment and size.
    pub fn assert_fits_within_storage(){
        let align_enum=std::mem::align_of::<E>();
        let align_storage=std::mem::align_of::<S>();
        assert!(
            Self::check_alignment(),
            "The alignment of the storage is lower than the enum:\n\t{}!={}",
            align_storage,align_enum,
        );
        let size_enum=std::mem::size_of::<E>();
        let size_storage=std::mem::size_of::<S>();
        assert!(
            Self::check_size(),
            "The size of the storage is smaller than the enum:\n\t{} < {}",
            size_storage,size_enum,
        );
    }
}

impl<E,S,I> NonExhaustive<E,S,I>
where
    E:GetEnumInfo
{
/**
Unwraps a reference to this `NonExhaustive<>` into a reference to the original enum.

# Errors

This returns an error if the wrapped enum is of a variant that is 
not valid in this context.

# Example

This shows NonExhaustive<enum> some of which can and cannot be unwrapped.
That enum come from a newer version of the library than this has an interface for.

```
use abi_stable::nonexhaustive_enum::{
    doc_enums::example_2::{Foo,new_a,new_b,new_c},
};

assert_eq!(new_a()  .as_enum().ok(),Some(&Foo::A)    );
assert_eq!(new_b(10).as_enum().ok(),Some(&Foo::B(10)));
assert_eq!(new_b(77).as_enum().ok(),Some(&Foo::B(77)));
assert_eq!(new_c().as_enum().ok()  ,None             );


```

*/
    pub fn as_enum(&self)->Result<&E,UnwrapEnumError<&Self>>{
        let discriminant=self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            unsafe{
                Ok(&*(&self.fill as *const ManuallyDrop<S> as *const E))
            }
        }else{
            Err(UnwrapEnumError::new(self))
        }
    }

/**
Unwraps a mutable reference to this `NonExhaustive<>` into a 
mutable reference to the original enum.

# Errors

This returns an error if the wrapped enum is of a variant that is 
not valid in this context.

# Example

This shows NonExhaustive<enum> some of which can and cannot be unwrapped.
That enum come from a newer version of the library than this has an interface for.

```
use abi_stable::nonexhaustive_enum::{
    doc_enums::example_1::{Foo,new_a,new_b,new_c},
};

assert_eq!(new_a()  .as_enum_mut().ok(),Some(&mut Foo::A));
assert_eq!(new_b(10).as_enum_mut().ok(),None);
assert_eq!(new_b(77).as_enum_mut().ok(),None);
assert_eq!(new_c().as_enum_mut().ok()  ,None);


```

*/
    pub fn as_enum_mut(&mut self)->Result<&mut E,UnwrapEnumError<&mut Self>>
    where 
        E:GetVTable<S,I>,
    {
        let discriminant=self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            /*
            Must update the vtable every time as_enum_mut is called,
            because if the enum is replaced with a variant with a discriminant
            outside the valid range for the functions in the vtable,
            it would be undefined behavior to call those functions.
            */
            self.vtable=E::VTABLE_REF;
            unsafe{
                Ok(&mut *(&mut self.fill as *mut ManuallyDrop<S> as *mut E))
            }
        }else{
            Err(UnwrapEnumError::new(self))
        }
    }

/**
Unwraps this `NonExhaustive<>` into the original enum.

# Errors

This returns an error if the wrapped enum is of a variant that is 
not valid in this context.

# Example

This shows NonExhaustive<enum> some of which can and cannot be unwrapped.
That enum come from a newer version of the library than this has an interface for.

```
use abi_stable::nonexhaustive_enum::{
    doc_enums::example_2::{Foo,new_a,new_b,new_c},
};

assert_eq!(new_a()  .into_enum().ok(),Some(Foo::A));
assert_eq!(new_b(10).into_enum().ok(),Some(Foo::B(10)));
assert_eq!(new_b(77).into_enum().ok(),Some(Foo::B(77)));
assert_eq!(new_c().into_enum().ok()  ,None);


*/
    pub fn into_enum(self)->Result<E,UnwrapEnumError<Self>>{
        let discriminant=self.get_discriminant();
        if E::is_valid_discriminant(discriminant) {
            let this=ManuallyDrop::new(self);
            unsafe{
                Ok((&this.fill as *const ManuallyDrop<S> as *const E).read())
            }
        }else{
            Err(UnwrapEnumError::new(self))
        }
    }

/**
Returns whether the discriminant of this enum is valid in this context.

The only way for it to be invalid is if the dynamic library is a 
newer version than the interface this has a dependency.
*/
    #[inline]
    pub fn is_valid_discriminant(&self)->bool{
        E::is_valid_discriminant(self.get_discriminant())
    }

/**
Gets the value of the discriminant of the enum.
*/
    #[inline]
    pub fn get_discriminant(&self)->E::Discriminant{
        unsafe{
            *(&self.fill as *const ManuallyDrop<S> as *const E::Discriminant)
        }
    }

}

impl<E,S,I> NonExhaustive<E,S,I>{

/**
Transmute this `NonExhaustive<E,S,I>` into `NonExhaustive<F,S,I>`,
changing the type of the enum it wraps.

# Safety

This has the same safety requirements that `std::mem::transmute` has.

# Panics

This panics if the storage has an alignment or size smaller than that of `F`.

*/
    pub unsafe fn transmute_enum<F>(self)->NonExhaustive<F,S,I>{
        NonExhaustive::<F,S,I>::assert_fits_within_storage();
        transmute_ignore_size(self)
    }

/**
Transmute this `&NonExhaustive<E,S,I>` into `&NonExhaustive<F,S,I>`,
changing the type of the enum it wraps.

# Safety

This has the same safety requirements that `std::mem::transmute` has.

# Panics

This panics if the storage has an alignment or size smaller than that of `F`.

*/
    pub unsafe fn transmute_enum_ref<F>(&self)->&NonExhaustive<F,S,I>{
        NonExhaustive::<F,S,I>::assert_fits_within_storage();
        mem::transmute(self)
    }


/**
Transmute this `&mut NonExhaustive<E,S,I>` into `&mut NonExhaustive<F,S,I>`,
changing the type of the enum it wraps.

# Safety

This has the same safety requirements that `std::mem::transmute` has.

# Panics

This panics if the storage has an alignment or size smaller than that of `F`.

*/
    pub unsafe fn transmute_enum_mut<F>(&mut self)->&mut NonExhaustive<F,S,I>{
        NonExhaustive::<F,S,I>::assert_fits_within_storage();
        mem::transmute(self)
    }

/**
Transmute this pointer to a `NonExhaustive<E,S,I>` into 
a pointer (of the same kind) to a `NonExhaustive<F,S,I>`,
changing the type of the enum it wraps.

# Safety

This has the same safety requirements that 
`abi_stable::pointer_traits::TransmuteElement::transmute_element` has.

# Panics

This panics if the storage has an alignment or size smaller than that of `F`.

*/

    pub unsafe fn transmute_enum_ptr<P,F>(this:P)->P::TransmutedPtr
    where
        P:Deref<Target=Self>,
        P:TransmuteElement<NonExhaustive<F,S,I>>
    {
        NonExhaustive::<F,S,I>::assert_fits_within_storage();
        this.transmute_element(<NonExhaustive<F,S,I>>::T)
    }

    /// Gets a reference to the vtable of this `NonExhaustive<>`.
    pub fn vtable<'a>(&self)->&'a NonExhaustiveVtable<E,S,I>{
        self.vtable.get()
    }

    fn as_ref_with_interface<I2>(&self)->&NonExhaustive<E,S,I2>{
        unsafe{
            mem::transmute::<&NonExhaustive<E,S,I>,&NonExhaustive<E,S,I2>>(self)
        }
    }

    fn sabi_erased_ref(&self)->&ErasedObject{
        unsafe{
            &*(&self.fill as *const ManuallyDrop<S> as *const ErasedObject)
        }
    }

    fn sabi_erased_mut(&mut self)->&mut ErasedObject{
        unsafe{
            &mut *(&mut self.fill as *mut ManuallyDrop<S> as *mut ErasedObject)
        }
    }
}


impl<E,S,I> Clone for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Clone = Implemented<trait_marker::Clone>>,
{
    fn clone(&self)->Self{
        self.vtable().clone_()(self.sabi_erased_ref(),self.vtable)
    }
}

impl<E,S,I> Display for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Display = Implemented<trait_marker::Display>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        c_functions::adapt_std_fmt::<ErasedObject>(
            self.sabi_erased_ref(),
            self.vtable().display(),
            f
        )
    }
}

impl<E,S,I> Debug for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Debug = Implemented<trait_marker::Debug>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        c_functions::adapt_std_fmt::<ErasedObject>(
            self.sabi_erased_ref(),
            self.vtable().debug(),
            f
        )
    }
}


impl<E,S,I> Eq for NonExhaustive<E,S,I>
where
    Self: PartialEq,
    I: InterfaceBound<Eq = Implemented<trait_marker::Eq>>,
{
}


impl<E,S,I1,I2> PartialEq<NonExhaustive<E,S,I2>> for NonExhaustive<E,S,I1>
where
    I1: InterfaceBound<PartialEq = Implemented<trait_marker::PartialEq>>,
{
    fn eq(&self, other: &NonExhaustive<E,S,I2>) -> bool {
        self.vtable().partial_eq()(self.sabi_erased_ref(), other.as_ref_with_interface())
    }
}


impl<E,S,I> Ord for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Ord = Implemented<trait_marker::Ord>>,
    Self: PartialOrd + Eq,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.vtable().cmp()(self.sabi_erased_ref(), other.as_ref_with_interface()).into()
    }
}


impl<E,S,I1,I2> PartialOrd<NonExhaustive<E,S,I2>> for NonExhaustive<E,S,I1>
where
    I1: InterfaceBound<PartialOrd = Implemented<trait_marker::PartialOrd>>,
    Self: PartialEq<NonExhaustive<E,S,I2>>,
{
    fn partial_cmp(&self, other: &NonExhaustive<E,S,I2>) -> Option<Ordering> {
        self.vtable().partial_cmp()(self.sabi_erased_ref(), other.as_ref_with_interface())
            .map(IntoReprRust::into_rust)
            .into()
    }
}



/////////////////////


impl<E,S,I> PartialOrd<E> for NonExhaustive<E,S,I>
where
    E: GetEnumInfo+PartialOrd,
    I: InterfaceBound<PartialOrd = Implemented<trait_marker::PartialOrd>>,
    Self: PartialEq<E>,
{
    fn partial_cmp(&self, other: &E) -> Option<Ordering> {
        match self.as_enum() {
            Ok(this)=>this.partial_cmp(other),
            Err(_)=>Some(Ordering::Greater),
        }
    }
}

impl<E,S,I> PartialEq<E> for NonExhaustive<E,S,I>
where
    E: GetEnumInfo+PartialEq,
    I: InterfaceBound<PartialEq = Implemented<trait_marker::PartialEq>>,
{
    fn eq(&self, other: &E) -> bool {
        match self.as_enum() {
            Ok(this)=>this==other,
            Err(_)=>false,
        }
    }
}


/////////////////////



impl<E,S,I> NonExhaustive<E,S,I>{
    /// It serializes a `NonExhaustive<_>` into a string by using 
    /// `<ConcreteType as SerializeImplType>::serialize_impl`.
    pub fn serialized<'a>(&'a self) -> Result<RCow<'a, str>, RBoxError>
    where
        I: SerializeEnum<E>+InterfaceBound<Serialize=Implemented<trait_marker::Serialize>>,
    {
        self.vtable().serialize()(self.sabi_erased_ref()).into_result()
    }

    /// Deserializes a string into a `NonExhaustive<_>`,by using 
    /// `<I as DeserializeOwned>::deserialize_enum`.
    pub fn deserialize_owned_from_str(s: &str) -> Result<Self, RBoxError>
    where
        I: DeserializeOwned<E,S,I>,
        I: InterfaceBound<Deserialize= Implemented<trait_marker::Deserialize>>,
        E:GetEnumInfo,
    {
        s.piped(RStr::from).piped(I::deserialize_enum)
    }

    /// Deserializes a `&'borr str` into a `NonExhaustive<'borr,_>`,by using 
    /// `<I as DeserializeBorrowed<'borr>>::deserialize_enum`.
    pub fn deserialize_borrowing_from_str<'borr>(s: &'borr str) -> Result<Self, RBoxError>
    where
        Self:'borr,
        I: DeserializeBorrowed<'borr,E,S,I>,
        I: InterfaceBound<Deserialize= Implemented<trait_marker::Deserialize>>,
        E:GetEnumInfo,
    {
        s.piped(RStr::from).piped(I::deserialize_enum)
    }

}

/**
First it serializes a `NonExhaustive<_>` into a string by using 
<ConcreteType as SerializeImplType>::serialize_impl,
then it serializes the string.

*/
impl<E,S,I> Serialize for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Serialize = Implemented<trait_marker::Serialize>>,
{
    fn serialize<Z>(&self, serializer: Z) -> Result<Z::Ok, Z::Error>
    where
        Z: Serializer,
    {
        self.vtable().serialize()(self.sabi_erased_ref())
            .into_result()
            .map_err(ser::Error::custom)?
            .serialize(serializer)
    }
}


/// First it Deserializes a string,then it deserializes into a 
/// `NonExhaustive<_>`,by using `<I as DeserializeOwned>::deserialize_impl`.
impl<'de,E,S,I> Deserialize<'de> for NonExhaustive<E,S,I>
where
    E: 'de+Deserialize<'de>+GetVTable<S,I>,
    S: 'de,
    I: 'de+InterfaceBound<Deserialize=Implemented<trait_marker::Deserialize>>,
    I: DeserializeOwned<E,S,I>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        I::deserialize_enum(RStr::from(&*s)).map_err(de::Error::custom)
    }
}




/////////////////////


impl<E,S,I> Hash for NonExhaustive<E,S,I>
where
    I: InterfaceBound<Hash = Implemented<trait_marker::Hash>>,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.vtable().hash()(self.sabi_erased_ref(), HasherObject::new(state))
    }
}


impl<E,S,I> std::error::Error for NonExhaustive<E,S,I>
where
    I: InterfaceBound<
        Debug = Implemented<trait_marker::Debug>,
        Display = Implemented<trait_marker::Display>,
        Error = Implemented<trait_marker::Error>
    >,
{}

/////////////////////


impl<E,S,I> Drop for NonExhaustive<E,S,I>{
    fn drop(&mut self){
        let drop=self.vtable()._sabi_drop();

        unsafe{
            drop(self.sabi_erased_mut());
        }
    }
}


///////////////////////////////////////////////////////////////////////////////

/// Used to abstract over the reference-ness of `NonExhaustive<>` inside UnwrapEnumError.
pub trait NonExhaustiveSharedOps{
    type Discriminant:ValidDiscriminant;
    fn get_discriminant_(&self)->Self::Discriminant;
    fn enum_info_(&self)->&'static EnumInfo<Self::Discriminant>;
}


macro_rules! impl_neso {
    (
        impl[$E:ident,$S:ident,$I:ident]
    ) => (
        type Discriminant=$E::Discriminant;

        fn get_discriminant_(&self)->$E::Discriminant {
            self.get_discriminant()
        }

        fn enum_info_(&self)->&'static EnumInfo<Self::Discriminant>{
            self.vtable().enum_info()
        }
    )
}


impl<E,S,I> NonExhaustiveSharedOps for NonExhaustive<E,S,I>
where
    E:GetEnumInfo,
{
    impl_neso!{ impl[E,S,I] }
}

impl<'a,E,S,I> NonExhaustiveSharedOps for &'a NonExhaustive<E,S,I>
where
    E:GetEnumInfo,
{
    impl_neso!{ impl[E,S,I] }
}

impl<'a,E,S,I> NonExhaustiveSharedOps for &'a mut NonExhaustive<E,S,I>
where
    E:GetEnumInfo,
{
    impl_neso!{ impl[E,S,I] }
}


///////////////////////////////////////////////////////////////////////////////


#[must_use]
#[repr(transparent)]
#[derive(Clone,PartialEq,Eq,PartialOrd,Ord,StableAbi)]
pub struct UnwrapEnumError<N>{
    pub non_exhaustive:N,
}



impl<N> UnwrapEnumError<N>{
    /// Extracts the `NonExhaustive<>`,to handle the failure to unwrap it.
    #[must_use]
    pub fn into_inner(self)->N{
        self.non_exhaustive
    }
}

impl<N> UnwrapEnumError<N>{
    #[inline]
    pub const fn new(non_exhaustive:N)->Self{
        Self{non_exhaustive}
    }
}

impl<N> Display for UnwrapEnumError<N>
where
    N:NonExhaustiveSharedOps,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        write!(
            f,
            "Could not unwrap NonExhaustive into '{}'.\n\
             Because its discriminant was {:?} .",
            self.non_exhaustive.enum_info_().type_name,
            self.non_exhaustive.get_discriminant_(),
        )
    }
}

impl<N> Debug for UnwrapEnumError<N>
where
    N:NonExhaustiveSharedOps,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("UnwrapEnumError")
         .field("non_exhaustive",&"<opaque>")
         .field("discriminant",&self.non_exhaustive.get_discriminant_())
         .field("enum",&self.non_exhaustive.enum_info_().type_name)
         .finish()
    }
}


impl<N> std::error::Error for UnwrapEnumError<N>
where
    N:NonExhaustiveSharedOps,
{}

