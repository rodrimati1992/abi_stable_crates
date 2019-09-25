
/**
Use this when constructing a `abi_stable::type_layout::MonoTypeLayout`
when manually implementing StableAbi.

This stores indices and ranges for the type and/or const parameter taken 
from the SharedVars of the TypeLayout where this is stored.

# Syntax

`tl_genparams!( (<lifetime>),* ; <convertible_to_startlen>; <convertible_to_startlen> )`

`<convertible_to_startlen>` is one of:

-``: No generic parameters of that kind.

-`i`: 
    Takes the ith generic parameter
    from the SharedVars slice of type or the one of const parameters.

-`i..j`: 
    Takes from i to j exclusive generic parameter
    from the SharedVars slice of type or the one of const parameters.

-`i..=j`: 
    Takes from i to j inclusive generic parameter
    from the SharedVars slice of type or the one of const parameters.

-`x:StartLen`: 
    Takes from x.start() to x.end() exclusive generic parameter
    from the SharedVars slice of type or the one of const parameters.



# Examples

### No generic parameters:

```
use abi_stable::{
    type_layout::CompGenericParams,
    tl_genparams,
};

const PARAMS:CompGenericParams=tl_genparams!(;;);

```

### One lifetime,one type parameter,one const parameter

```
use abi_stable::{
    type_layout::CompGenericParams,
    tl_genparams,
};

const PARAMS:CompGenericParams=tl_genparams!('a;0;0);

```

### One lifetime,two type parameters,no const parameters

```
use abi_stable::{
    type_layout::CompGenericParams,
    tl_genparams,
};

const PARAMS:CompGenericParams=tl_genparams!('a;0..=1;);

```

### Four lifetimes,no type parameters,three const parameters

```
use abi_stable::{
    type_layout::CompGenericParams,
    tl_genparams,
};

const PARAMS:CompGenericParams=tl_genparams!('a,'b,'c,'d;;0..3);

```

### No lifetimes,two type parameters,no const parameters

```
use abi_stable::{
    type_layout::{CompGenericParams,StartLen},
    tl_genparams,
};

const PARAMS:CompGenericParams=tl_genparams!(;StartLen::new(0,2););

```



*/
#[macro_export]
macro_rules! tl_genparams {
    (count;  ) => ( 0 );
    (count; $v0:lifetime) => ( 1 );
    (count; $v0:lifetime,$v1:lifetime $(,$rem:lifetime)*) => ( 
        2 + $crate::tl_genparams!(count; $($rem),* )
    );
    ( $($lt:lifetime),* $(,)? ; $($ty:expr)? ; $($const_p:expr)? ) => ({
        #[allow(unused_imports)]
        use $crate::{
            abi_stability::stable_abi_trait::SharedStableAbi,
            type_layout::CompGenericParams,
        };

        #[allow(unused_parens)]
        let ty_param_range=$crate::type_layout::StartLenConverter( ($($ty)?) ).to_start_len();
        
        #[allow(unused_parens)]
        let const_param_range=
            $crate::type_layout::StartLenConverter( ($($const_p)?) ).to_start_len();

        CompGenericParams::new(
            $crate::nul_str!($(stringify!($lt),",",)*),
            $crate::tl_genparams!(count; $($lt),* ),
            ty_param_range,
            const_param_range,
        )
    })
}


///////////////////////////////////////////////////////////////////////

/**
Equivalent to `?` for `RResult<_,_>`.

# Example

Defining an extern function that returns a result.

```
use abi_stable::{
    std_types::{RResult,ROk,RErr,RBoxError,RStr,Tuple3},
    traits::IntoReprC,
    rtry,
    sabi_extern_fn,
};


#[sabi_extern_fn]
fn parse_tuple(s:RStr<'_>)->RResult<Tuple3<u32,u32,u32>,RBoxError>{
    let mut iter=s.split(',').map(|x|x.trim());
    ROk(Tuple3(
        rtry!( iter.next().unwrap_or("").parse().map_err(RBoxError::new).into_c() ),
        rtry!( iter.next().unwrap_or("").parse().map_err(RBoxError::new).into_c() ),
        rtry!( iter.next().unwrap_or("").parse().map_err(RBoxError::new).into_c() ),
    ))
}



```

*/
#[macro_export]
macro_rules! rtry {
    ($expr:expr) => {{
        use $crate::std_types::result::{RErr, ROk};
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
        use $crate::std_types::option::{RNone, RSome};
        match $expr.into() {
            RSome(x) => x,
            RNone => return RNone,
        }
    }};
}



///////////////////////////////////////////////////////////////////////

macro_rules! check_unerased {
    (
        $this:ident,$res:expr
    ) => (
        if let Err(e)=$res {
            return Err( e.map(move|_| $this ) );
        }
    )
}


///////////////////////////////////////////////////////////////////////


/**
Use this to make sure that you handle panics inside `extern fn` correctly.

This macro causes an abort if a panic reaches this point.

It does not prevent functions inside from using `::std::panic::catch_unwind` to catch the panic.

# Early returns

This macro by default wraps the passed code in a closure so that any 
early returns that happen inside don't interfere with the macro generated code.

If you don't have an early return (a `return`/`continue`/`break`) 
in the code passed to this macro you can use 
`extern_fn_panic_handling!{no_early_return; <code here> }`,
which *might* be cheaper(this has not been tested yet).

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

# Example, no_early_return


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
    extern_fn_panic_handling!{no_early_return;
        use std::fmt::Write;

        println!("{:?}",this);
    }
}

```


*/
#[macro_export]
macro_rules! extern_fn_panic_handling {
    (no_early_return; $($fn_contents:tt)* ) => ({
        let aborter_guard={
            use $crate::utils::{AbortBomb,PanicInfo};
            #[allow(dead_code)]
            const BOMB:AbortBomb=AbortBomb{
                fuse:&PanicInfo{file:file!(),line:line!()}
            };
            BOMB
        };
        let res={
            $($fn_contents)*
        };

        ::std::mem::forget(aborter_guard);
        
        res
    });
    ( $($fn_contents:tt)* ) => (
        $crate::extern_fn_panic_handling!{
            no_early_return;
            let a=$crate::marker_type::NotCopyNotClone;
            (move||{
                drop(a);
                $($fn_contents)*
            })()
        }
    )
}


///////////////////////////////////////////////////////////////////////



/**

Constructs the abi_stable::erased_types::TypeInfo for some type.

It's necessary for the type to be `'static` because this uses UTypeId.

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
                std_types::{StaticStr,utypeid::some_utypeid},
            };

            $crate::impl_get_typename!{ let type_name= $type $([$($params)*])? }
            
            &TypeInfo{
                size:mem::size_of::<Self>(),
                alignment:mem::align_of::<Self>(),
                _uid:$crate::sabi_types::Constructor( some_utypeid::<Self> ),
                type_name,
                module:StaticStr::new(module_path!()),
                package:StaticStr::new(env!("CARGO_PKG_NAME")),
                package_version:$crate::package_version_strings!(),
                _private_field:(),
            }
        }
    )
}

#[macro_export]
#[cfg(not(any(rust_1_38,feature="rust_1_38")))]
#[doc(hidden)]
macro_rules! impl_get_typename{
    (
        let $type_name:ident = $type:ident $([$($params:tt)*])?
    ) => (
        let $type_name={
            extern "C" fn __get_type_name()->$crate::std_types::RStr<'static>{
                stringify!($type).into()
            }

            $crate::sabi_types::Constructor(__get_type_name)
        };
    )
}

#[macro_export]
#[cfg(any(rust_1_38,feature="rust_1_38"))]
#[doc(hidden)]
macro_rules! impl_get_typename{
    (
        let $type_name:ident = $type:ident $([$($params:tt)*])?
    ) => (
        let $type_name=$crate::sabi_types::Constructor(
            $crate::utils::get_type_name::<$type<$( $($params)* )? >>
        );
    )
}



///////////////////////////////////////////////////////////////////////////////////////////



/**
Constructs a abi_stable::abi_stability::Tag,
a dynamically typed value for users to check extra properties about their types 
when doing runtime type checking.

Note that this macro is not recursive,
you need to invoke it every time you construct an array/map/set inside of the macro.

For more examples look in the [tagging module](./abi_stability/tagging/index.html)

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
    type_layout::Tag,
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
        use $crate::type_layout::tagging::{Tag,FromLiteral};
        
        Tag::arr($crate::rslice![
            $( FromLiteral($elem).to_tag(), )*
        ])
    }};
    ({ $( $key:expr=>$value:expr ),* $(,)? })=>{{
        use $crate::type_layout::tagging::{FromLiteral,Tag};

        Tag::map($crate::rslice![
            $(
                Tag::kv(
                    FromLiteral($key).to_tag(),
                    FromLiteral($value).to_tag(),
                ),
            )*
        ])
    }};
    ({ $( $key:expr ),* $(,)? })=>{{
        use $crate::type_layout::tagging::{Tag,FromLiteral};

        Tag::set($crate::rslice![
            $(
                FromLiteral($key).to_tag(),
            )*
        ])
    }};
    ($expr:expr) => {{
        $crate::type_layout::tagging::FromLiteral($expr).to_tag()
    }};
}


///////////////////////////////////////////////////////////////////////////////


#[allow(unused_macros)]
macro_rules! assert_matches {
    ( $(|)* $pat:pat $(| $prev_pat:pat)*  =$expr:expr)=>{{
        let ref value=$expr;
        assert!(
            core_extensions::matches!($pat $(| $prev_pat)* = *value), 
            "pattern did not match the value:\n\t\
             {:?}
            ",
            *value
        );
    }};
}


///////////////////////////////////////////////////////////////////////////////


/**
Constructs a abi_stable::type_layout::ItemInfo,
with information about the place where it's called.
*/
#[macro_export]
macro_rules! make_item_info {
    () => (
        $crate::type_layout::ItemInfo::new(
            concat!(
                env!("CARGO_PKG_NAME"),
                ";",
                env!("CARGO_PKG_VERSION")
            ),
            line!(),
            $crate::type_layout::ModPath::inside($crate::nul_str!(module_path!())),
        )
    )
}


///////////////////////////////////////////////////////////////////////////////



/**
Constructs an `RVec<_>` using the same syntax that the `std::vec` macro uses.

# Example

```

use abi_stable::{
    rvec,
    std_types::RVec,  
};

assert_eq!(RVec::<u32>::new(), rvec![]);
assert_eq!(RVec::from(vec![0]), rvec![0]);
assert_eq!(RVec::from(vec![0,3]), rvec![0,3]);
assert_eq!(RVec::from(vec![0,3,6]), rvec![0,3,6]);
assert_eq!(RVec::from(vec![1;10]), rvec![1;10]);

```
*/
#[macro_export]
macro_rules! rvec {
    ( $( $anything:tt )* ) => (
        $crate::std_types::RVec::from(vec![ $($anything)* ])
    )
}


///////////////////////////////////////////////////////////////////////////////


/**
Use this macro to construct a `Tuple*` with the values passed to the macro.

# Example 

```
use abi_stable::{
    rtuple,
    std_types::{Tuple1,Tuple2,Tuple3,Tuple4},
};

assert_eq!(rtuple!(), ());

assert_eq!(rtuple!(3), Tuple1(3));

assert_eq!(rtuple!(3,5), Tuple2(3,5));

assert_eq!(rtuple!(3,5,8), Tuple3(3,5,8));

assert_eq!(rtuple!(3,5,8,9), Tuple4(3,5,8,9));

```

*/

#[macro_export]
macro_rules! rtuple {
    () => (());
    ($v0:expr $(,)* ) => (
        $crate::std_types::Tuple1($v0)
    );
    ($v0:expr,$v1:expr $(,)* ) => (
        $crate::std_types::Tuple2($v0,$v1)
    );
    ($v0:expr,$v1:expr,$v2:expr $(,)* ) => (
        $crate::std_types::Tuple3($v0,$v1,$v2)
    );
    ($v0:expr,$v1:expr,$v2:expr,$v3:expr $(,)* ) => (
        $crate::std_types::Tuple4($v0,$v1,$v2,$v3)
    );
}


/**
Use this macro to get the type of a `Tuple*` with the types passed to the macro.

# Example 

```
use abi_stable::{
    RTuple,
    std_types::{Tuple1,Tuple2,Tuple3,Tuple4},
};

let tuple0:RTuple!()=();

let tuple1:RTuple!(i32)=Tuple1(3);

let tuple2:RTuple!(i32,i32,)=Tuple2(3,5);

let tuple3:RTuple!(i32,i32,u32,)=Tuple3(3,5,8);

let tuple4:RTuple!(i32,i32,u32,u32)=Tuple4(3,5,8,9);


```

*/
#[macro_export]
macro_rules! RTuple {
    () => (
        ()
    );
    ($v0:ty $(,)* ) => (
        $crate::std_types::Tuple1<$v0>
    );
    ($v0:ty,$v1:ty $(,)* ) => (
        $crate::std_types::Tuple2<$v0,$v1>
    );
    ($v0:ty,$v1:ty,$v2:ty $(,)* ) => (
        $crate::std_types::Tuple3<$v0,$v1,$v2>
    );
    ($v0:ty,$v1:ty,$v2:ty,$v3:ty $(,)* ) => (
        $crate::std_types::Tuple4<$v0,$v1,$v2,$v3>
    );
}


///////////////////////////////////////////////////////////////////////////////

/**
A macro to construct `RSlice<'_,T>` constants.

# Examples

```
use abi_stable::{
    std_types::RSlice,
    rslice,
};

const RSLICE_0:RSlice<'static,u8>=rslice![];
const RSLICE_1:RSlice<'static,u8>=rslice![1];
const RSLICE_2:RSlice<'static,u8>=rslice![1,2];
const RSLICE_3:RSlice<'static,u8>=rslice![1,2,3];
const RSLICE_4:RSlice<'static,u8>=rslice![1,2,3,5];
const RSLICE_5:RSlice<'static,u8>=rslice![1,2,3,5,8];
const RSLICE_6:RSlice<'static,u8>=rslice![1,2,3,5,8,13];

assert_eq!( RSLICE_0.as_slice(), <&[u8]>::default() );
assert_eq!( RSLICE_0.len(), 0 );

assert_eq!( RSLICE_1.as_slice(), &[1] );
assert_eq!( RSLICE_1.len(), 1 );

assert_eq!( RSLICE_2.as_slice(), &[1,2] );
assert_eq!( RSLICE_2.len(), 2 );

assert_eq!( RSLICE_3.as_slice(), &[1,2,3] );
assert_eq!( RSLICE_3.len(), 3 );

assert_eq!( RSLICE_4.as_slice(), &[1,2,3,5] );
assert_eq!( RSLICE_4.len(), 4 );

assert_eq!( RSLICE_5.as_slice(), &[1,2,3,5,8] );
assert_eq!( RSLICE_5.len(), 5 );

assert_eq!( RSLICE_6.as_slice(), &[1,2,3,5,8,13] );
assert_eq!( RSLICE_6.len(), 6 );


```

*/
#[macro_export]
macro_rules! rslice {
    (@count;  ) => ( 0 );
    (@count; $v0:expr) => ( 1 );
    (@count; $v0:expr,$v1:expr) => ( 2 );
    (@count; $v0:expr,$v1:expr,$v2:expr) => ( 3 );
    (@count; $v0:expr,$v1:expr,$v2:expr,$v3:expr) => ( 4 );
    (@count; $v0:expr,$v1:expr,$v2:expr,$v3:expr $(,$rem:expr)+) => ( 
        4 + $crate::rslice!(@count; $($rem),* )
    );
    () => (
        $crate::std_types::RSlice::EMPTY
    );
    ( $( $elem:expr ),* $(,)* ) => (
        unsafe{
            // This forces the length to be evaluated at compile-time.
            const _RSLICE_LEN:usize=$crate::rslice!(@count; $($elem),* );
            $crate::std_types::RSlice::from_raw_parts_with_lifetime(
                &[ $($elem),* ],
                _RSLICE_LEN,
            )
        }
    );
}


///////////////////////////////////////////////////////////////////////////////



/**
A macro to construct `RStr<'static>` constants.

# Examples

```
use abi_stable::{
    std_types::RStr,
    rstr,
};

const RSTR_0:RStr<'static>=rstr!("");
const RSTR_1:RStr<'static>=rstr!("1");
const RSTR_2:RStr<'static>=rstr!("12");
const RSTR_3:RStr<'static>=rstr!("123");
const RSTR_4:RStr<'static>=rstr!("1235");
const RSTR_5:RStr<'static>=rstr!("12358");
const RSTR_6:RStr<'static>=rstr!("1235813");

assert_eq!( RSTR_0.as_str(), "" );
assert_eq!( RSTR_0.len(), 0 );

assert_eq!( RSTR_1.as_str(), "1" );
assert_eq!( RSTR_1.len(), 1 );

assert_eq!( RSTR_2.as_str(), "12" );
assert_eq!( RSTR_2.len(), 2 );

assert_eq!( RSTR_3.as_str(), "123" );
assert_eq!( RSTR_3.len(), 3 );

assert_eq!( RSTR_4.as_str(), "1235" );
assert_eq!( RSTR_4.len(), 4 );

assert_eq!( RSTR_5.as_str(), "12358" );
assert_eq!( RSTR_5.len(), 5 );

assert_eq!( RSTR_6.as_str(), "1235813" );
assert_eq!( RSTR_6.len(), 7 );


```

*/
#[macro_export]
macro_rules! rstr {
    ( $lit:literal ) => {unsafe{
        mod string_module{
            $crate::get_string_length!{$lit}
        }

        $crate::std_types::RStr::from_raw_parts(
            $lit.as_ptr(),
            string_module::LEN,
        )
    }}
}


/**
Constructs a NulStr from a string literal.

# Example

```
use abi_stable::{
    sabi_types::NulStr,
    nul_str,
};

assert_eq!( nul_str!("Huh?").to_str_with_nul(), "Huh?\0" );
assert_eq!( nul_str!("Hello!").to_str_with_nul(), "Hello!\0" );


```
*/
#[macro_export]
macro_rules! nul_str {
    ( $($str:expr),* $(,)* ) => {unsafe{
        $crate::sabi_types::NulStr::from_str(concat!($($str,)* "\0"))
    }}
}


/**
Constructs a RStr with the concatenation of the passed in strings,
and variables with the range for the individual strings.
*/
macro_rules! multi_str {
    (
        $( #[$mod_attr:meta] )*
        mod $module:ident {
            $( const $variable:ident=$string:literal; )*
        }
    ) => (
        $( #[$mod_attr] )*
        mod $module{
            $crate::abi_stable_derive::concatenated_and_ranges!{
                CONCATENATED( $($variable=$string),* )
            }
        }
    )
}

/**
Constructs a `&'static SharedVars`
*/
macro_rules! make_shared_vars{
    (
        let ($mono_shared_vars:ident,$shared_vars:ident) ={
            $(
                strings={
                     $( $variable:ident : $string:literal ),* $(,)*
                },
            )?
            $( lifetime_indices=[ $($lifetime_indices:expr),* $(,)* ], )?
            $( type_layouts=[ $($ty_layout:ty),* $(,)* ], )?
            $( type_layouts_shared=[ $($ty_layout_shared:ty),* $(,)* ], )?
            $( constants=[ $( $constants:expr ),* $(,)* ], )?
        };
    )=>{
        
        multi_str!{
            #[allow(non_upper_case_globals)]
            mod _inner_multi_str_mod{
                $( $( const $variable = $string; )* )?
            }
        }
        $( use _inner_multi_str_mod::{$($variable,)*}; )?

        const $mono_shared_vars:&'static $crate::type_layout::MonoSharedVars=
            &$crate::type_layout::MonoSharedVars::new(
                _inner_multi_str_mod::CONCATENATED,
                rslice![ $( $($lifetime_indices),* )? ],
            );
        
        let $shared_vars={
            #[allow(unused_imports)]
            use $crate::abi_stability::stable_abi_trait::GetTypeLayoutCtor;

            &$crate::type_layout::SharedVars::new(
                $mono_shared_vars,
                rslice![ 
                    $( $( GetTypeLayoutCtor::<$ty_layout>::STABLE_ABI,)* )? 
                    $( $( GetTypeLayoutCtor::<$ty_layout_shared>::SHARED_STABLE_ABI,)* )? 
                ],
                rslice![$( 
                    $(
                        $crate::abi_stability::ConstGeneric::new(
                            &$constants,
                            $crate::abi_stability::GetConstGenericVTable::VTABLE,
                        )
                    ),* 
                )?],
            )
        };
    }
}


///////////////////////////////////////////////////////////////////////////////


#[allow(unused_macros)]
macro_rules! delegate_interface_serde {
    (
        impl[$($impl_header:tt)* ] Traits<$this:ty> for $interf:ty ;
        lifetime=$lt:lifetime;
        delegate_to=$delegates_to:ty;
    ) => (
        impl<$($impl_header)*> $crate::nonexhaustive_enum::SerializeEnum<$this> for $interf
        where
            $delegates_to:
                $crate::nonexhaustive_enum::SerializeEnum<$this>
        {
            type Proxy=<
                $delegates_to as
                $crate::nonexhaustive_enum::SerializeEnum<$this>
            >::Proxy;

            fn serialize_enum<'a>(
                this:&'a $this
            ) -> Result<Self::Proxy, $crate::std_types::RBoxError>{
                <$delegates_to>::serialize_enum(this)
            }
        }

        impl<$lt,$($impl_header)* S,I> 
            $crate::nonexhaustive_enum::DeserializeEnum<
                $lt,
                $crate::nonexhaustive_enum::NonExhaustive<$this,S,I>
            > 
        for $interf
        where
            $this:$crate::nonexhaustive_enum::GetEnumInfo+$lt,
            $delegates_to:
                $crate::nonexhaustive_enum::DeserializeEnum<
                    $lt,
                    $crate::nonexhaustive_enum::NonExhaustive<$this,S,I>
                >
        {
            type Proxy=<
                $delegates_to as
                $crate::nonexhaustive_enum::DeserializeEnum<
                    $lt,
                    $crate::nonexhaustive_enum::NonExhaustive<$this,S,I>
                >
            >::Proxy;

            fn deserialize_enum(
                s: Self::Proxy
            ) -> Result<
                    $crate::nonexhaustive_enum::NonExhaustive<$this,S,I>,
                    $crate::std_types::RBoxError
                >
            {
                <$delegates_to as 
                    $crate::nonexhaustive_enum::DeserializeEnum<
                        $crate::nonexhaustive_enum::NonExhaustive<$this,S,I>
                    >
                >::deserialize_enum(s)
            }
        }
    )
}


