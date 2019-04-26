
/**
Use this when constructing a `abi_stable::abi_stability::TypeLayoutParams`
when manually implementing StableAbi,
to more ergonomically initialize the generics field.

# Example 

A type with lifetime type,and lifetime parameters:

```
use abi_stable::{
    tl_genparams,
    StableAbi,
    abi_stability::type_layout::GenericParams
};

struct Reference<'a,'b,T,U>(&'a T,&'b U);

impl<'a,'b,T,U> Reference<'a,'b,T,U>
where
    T:StableAbi,
    U:StableAbi,
{
    const GENERICS:GenericParams=
        tl_genparams!('a,'b ; T,U ; );
}


```

# Example

A type with lifetime,type,and const parameters.

Note that while this example won't compile until const parameters are usable,

```ignore
use abi_stable::{
    tl_genparams,
    StableAbi,
    abi_stability::type_layout::GenericParams
};

struct ArrayReference<'a,'b,T,U,const SIZE_T:usize,const SIZE_U:usize>{
    first:&'a [T;SIZE_T],
    second:&'b [U;SIZE_U],
}

impl<'a,'b,T,U,const SIZE_T:usize,const SIZE_U:usize> 
    Reference<'a,'b,T,U,SIZE_T,SIZE_U>
where
    T:StableAbi,
    U:StableAbi,
{
    const GENERICS:GenericParams=
        tl_genparams!('a,'b ; T,U ; SIZE_T,SIZE_U );
}

```



*/
#[macro_export]
macro_rules! tl_genparams {
    ( $($lt:lifetime),*  ; $($ty:ty),*  ; $($const_p:expr),*  ) => ({
        #[allow(unused_imports)]
        use $crate::{
            abi_stability::{
                stable_abi_trait::SharedStableAbi,
                type_layout::GenericParams
            },
            std_types::StaticStr,
        };

        GenericParams::new(
            &[$( StaticStr::new( stringify!($lt) ) ,)*],
            &[$( <$ty as SharedStableAbi>::S_LAYOUT ,)*],
            &[$( StaticStr::new( stringify!($const_p) ) ,)*],
        )
    })
}




///////////////////////////////////////////////////////////////////////


/// Equivalent to `?` for `RResult<_,_>`.
#[macro_export]
macro_rules! rtry {
    ($expr:expr) => {{
        use $crate::result::{RErr, ROk};
        match $expr.into() {
            ROk(x) => x,
            RErr(x) => return RErr(From::from(x)),
        }
    }};
}

/// Equivalent to `?` for `ROption<_>`.
#[macro_export]
macro_rules! rtry_opt {
    ($expr:expr) => {{
        use $crate::option::{RNone, RSome};
        match $expr.into() {
            RSome(x) => x,
            RNone => return RNone,
        }
    }};
}



///////////////////////////////////////////////////////////////////////


macro_rules! make_rve_utypeid {
    ($ty:ty) => (
        $crate::return_value_equality::ReturnValueEquality{
            function:$crate::std_types::utypeid::new_utypeid::<$ty>
        }
    )
}



///////////////////////////////////////////////////////////////////////




/**
Use this to make sure that you handle panics inside `extern fn` correctly.

This macro causes an abort if a panic reaches this point.

It does not prevent functions inside from using `::std::panic::catch_unwind` to catch the panic.

# Example 

```
use std::fmt;

use abi_stable::{
    extern_fn_panic_handling,
    std_types::RString,
};


pub extern "C" fn print_debug<T>(this: &T,buf: &mut RString)
where
    T: fmt::Debug,
{
    extern_fn_panic_handling! {
        use std::fmt::Write;

        println!("{:?}",this);
    }
}


```


*/
#[macro_export]
macro_rules! extern_fn_panic_handling {
    ( $($fn_contents:tt)* ) => ({
        use std::panic::{self,AssertUnwindSafe};

        let result=panic::catch_unwind(AssertUnwindSafe(move||{
            $($fn_contents)*
        }));

        match result {
            Ok(x)=>x,
            Err(_)=>$crate::utils::ffi_panic_message(file!(),line!()),
        }
    })
}


///////////////////////////////////////////////////////////////////////



/**

Implements the abi_stable::type_info::GetTypeInfo trait for some type.

It's necessary for the type to be `'static` because this uses TypeId.

# Example

```
use abi_stable::{
    impl_get_type_info,
    erased_types::{TypeInfo,ImplType},
};

#[derive(Default, Clone, Debug)]
struct Foo<T> {
    l: u32,
    r: u32,
    name: T,
}

impl<T> ImplType for Foo<T>
where T:'static+Send+Sync
{
    type Interface=();
    
    // You have to write the full type (eg: impl_get_type_info!{ Bar['a,T,U] } ) ,
    // never write Self.
    const INFO:&'static TypeInfo=impl_get_type_info! { Foo[T] };
}

```

*/
#[macro_export]
macro_rules! impl_get_type_info {
    (
        $type:ident $([$($params:tt)*])?
    ) => (
        {
            use std::mem;
            use $crate::{
                erased_types::type_info::TypeInfo,
                version::{VersionStrings},
                std_types::{StaticStr,utypeid::new_utypeid},
                return_value_equality::ReturnValueEquality,
            };

            &TypeInfo{
                size:mem::size_of::<Self>(),
                alignment:mem::align_of::<Self>(),
                uid:ReturnValueEquality{
                    function:new_utypeid::<Self>
                },
                name:StaticStr::new(stringify!($type)),
                file:StaticStr::new(file!()),
                package:StaticStr::new(env!("CARGO_PKG_NAME")),
                package_version:VersionStrings{
                    major:StaticStr::new(env!("CARGO_PKG_VERSION_MAJOR")),
                    minor:StaticStr::new(env!("CARGO_PKG_VERSION_MINOR")),
                    patch:StaticStr::new(env!("CARGO_PKG_VERSION_PATCH")),
                },
                _private_field:(),
            }
        }
    )
}


///////////////////////////////////////////////////////////////////////////////////////////



/**
Constructs a abi_stable::abi_stability::Tag,
a dynamically typed value for users to check extra properties about their types 
when doing runtime type checking.

Note that this macro is not recursive,
you need to invoke it every time you construct an array/map/set inside of the macro.

# Example

Using tags to store the traits the type requires,
so that if this changes it can be reported as an error.

This will cause an error if the binary and dynamic library disagree about the values inside
the "required traits" map entry .

In real code this should be written in a 
way that keeps the tags and the type bounds in sync.


```
use abi_stable::{
    tag,
    abi_stability::Tag,
    StableAbi,
};

const TAGS:Tag=tag!{{
    "required traits"=>tag![[ "Copy" ]],
}};


#[repr(C)]
#[derive(StableAbi)]
#[sabi(bound="T:Copy")]
#[sabi(tag="TAGS")]
struct Value<T>{
    value:T,
}


```

*/
#[macro_export]
macro_rules! tag {
    ([ $( $elem:expr ),* $(,)? ])=>{{
        use $crate::abi_stability::tagging::FromLiteral;
        
        Tag::arr(&[
            $( FromLiteral($elem).to_tag(), )*
        ])
    }};
    ({ $( $key:expr=>$value:expr ),* $(,)? })=>{{
        use $crate::abi_stability::tagging::{FromLiteral,Tag};

        Tag::map(&[
            $(
                Tag::kv(
                    FromLiteral($key).to_tag(),
                    FromLiteral($value).to_tag(),
                ),
            )*
        ])
    }};
    ({ $( $key:expr ),* $(,)? })=>{{
        use $crate::abi_stability::tagging::FromLiteral;

        Tag::set(&[
            $(
                FromLiteral($key).to_tag(),
            )*
        ])
    }};
    ($expr:expr) => {{
        $crate::abi_stability::tagging::FromLiteral($expr).to_tag()
    }};
}