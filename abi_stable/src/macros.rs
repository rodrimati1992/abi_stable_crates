
/**
Use this when constructing a [`MonoTypeLayout`] when manually implementing StableAbi.

This stores indices and ranges for the type and/or const parameter taken 
from the [`SharedVars`] stored in the same [`TypeLayout`] where this is stored.

# Syntax

`tl_genparams!( (<lifetime>),* ; <convertible_to_startlen>; <convertible_to_startlen> )`

`<convertible_to_startlen>` is one of:

-` `: No generic parameters of that kind.

-`i`: 
    Takes the ith type or const parameter(indexed separately).

-`i..j`: 
    Takes from i to j (exclusive) type or const parameter(indexed separately).

-`i..=j`: 
    Takes from i to j (inclusive) type or const parameter(indexed separately).

-`x:StartLen`: 
    Takes from x.start() to x.end() (exclusive) type or const parameter(indexed separately).



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

[`MonoTypeLayout`]: ./type_layout/struct.MonoTypeLayout.html
[`TypeLayout`]: ./type_layout/struct.TypeLayout.html
[`SharedVars`]: ./type_layout/struct.SharedVars.html

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
Equivalent to `?` for [`RResult`].

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

[`RResult`]: ./std_types/enum.RResult.html

*/
#[macro_export]
macro_rules! rtry {
    ($expr:expr) => {{
        use $crate::std_types::{RErr, ROk};
        match $expr.into() {
            ROk(x) => x,
            RErr(x) => return RErr(From::from(x)),
        }
    }};
}

/// Equivalent to `?` for [`ROption`].
/// 
/// [`ROption`]: ./std_types/enum.ROption.html
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

Constructs the [`TypeInfo`] for some type.

It's necessary for the type to be `'static` because 
[`TypeInfo`] stores a private function that returns the  [`UTypeId`] of that type.

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
    
    // You have to write the full type (eg: impl_get_type_info!{ Bar<'a,T,U> } ) ,
    // never write Self.
    const INFO:&'static TypeInfo=impl_get_type_info! { Foo<T> };
}


```

[`TypeInfo`]: ./erased_types/struct.TypeInfo.html
[`UTypeId`]: ./std_types/struct.UTypeId.html

*/
#[macro_export]
macro_rules! impl_get_type_info {
    ($type:ty) => (
        {
            use std::mem;
            use $crate::{
                erased_types::TypeInfo,
                std_types::{RStr,utypeid::some_utypeid},
            };

            let type_name=$crate::sabi_types::Constructor($crate::utils::get_type_name::<$type>);
            
            &TypeInfo{
                size:mem::size_of::<Self>(),
                alignment:mem::align_of::<Self>(),
                _uid:$crate::sabi_types::Constructor( some_utypeid::<Self> ),
                type_name,
                module:RStr::from_str(module_path!()),
                package:RStr::from_str(env!("CARGO_PKG_NAME")),
                package_version:$crate::package_version_strings!(),
                _private_field:(),
            }
        }
    )
}

///////////////////////////////////////////////////////////////////////////////////////////



/**
Constructs a [`Tag`],
a dynamically typed value for users to check extra properties about their types 
when doing runtime type checking.

Note that this macro is not recursive,
you need to invoke it every time you construct an array/map/set inside of the macro.

For more examples look in the [tagging module](./type_layout/tagging/index.html)

# Example

Using tags to store the traits the type requires,
so that if this changes it can be reported as an error.

This will cause an error if the binary and dynamic library disagree about the values inside
the "required traits" map entry .

In real code this should be written in a 
way that keeps the tags and the type bounds in sync.


*/
#[cfg_attr(not(feature = "no_fn_promotion"), doc = "```rust")]
#[cfg_attr(feature = "no_fn_promotion", doc = "```ignore")]
/**
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

[`Tag`]: ./type_layout/tagging/struct.Tag.html

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
Constructs an [`ItemInfo`], with information about the place where it's called.

[`ItemInfo`]: ./type_layout/struct.ItemInfo.html
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
Constructs an [`RVec`] using the same syntax that the `std::vec` macro uses.

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

[`RVec`]: ./std_types/struct.RVec.html

*/
#[macro_export]
macro_rules! rvec {
    ( $( $anything:tt )* ) => (
        $crate::std_types::RVec::from(vec![ $($anything)* ])
    )
}


///////////////////////////////////////////////////////////////////////////////


/**
Use this macro to construct a `abi_stable::std_types::Tuple*`
with the values passed to the macro.

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
A macro to construct [`RSlice`] constants.

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

[`RSlice`]: ./std_types/struct.RSlice.html

*/
#[macro_export]
macro_rules! rslice {
    ( $( $elem:expr ),* $(,)* ) => {
        $crate::std_types::RSlice::from_slice(&[ $($elem),* ])
    };
}


///////////////////////////////////////////////////////////////////////////////


/**
A macro to construct [`RStr`] constants.

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

[`RStr`]: ./std_types/struct.RStr.html

*/
#[macro_export]
macro_rules! rstr{
    ( $str:expr ) => ({
        const __SABI_RSTR: $crate::std_types::RStr<'static> = 
            $crate::std_types::RStr::from_str($str);
        __SABI_RSTR
    })
}


/**
Constructs a [`NulStr`] from a string literal.

[`NulStr`]: ./sabi_types/struct.NulStr.html

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
        impl[$($impl_gen:tt)*] $type:ty
        $(where[$($where_clause:tt)*])?;

        let ($mono_shared_vars:ident,$shared_vars:ident) ={
            $(
                strings={
                     $( $variable:ident : $string:literal ),* $(,)*
                },
            )?
            $( lifetime_indices=[ $($lifetime_indices:expr),* $(,)* ], )?
            $( type_layouts=[ $($ty_layout:ty),* $(,)* ], )?
            $( prefix_type_layouts=[ $($prefix_ty_layout:ty),* $(,)* ], )?
            $( constant=[ $const_ty:ty => $constants:expr  ], )?
        };
    )=>{
        multi_str!{
            #[allow(non_upper_case_globals)]
            mod _inner_multi_str_mod{
                $( $( const $variable = $string; )* )?
            }
        }
        $( use _inner_multi_str_mod::{$($variable,)*}; )?

        #[allow(non_upper_case_globals)]
        const $mono_shared_vars:&'static $crate::type_layout::MonoSharedVars=
            &$crate::type_layout::MonoSharedVars::new(
                _inner_multi_str_mod::CONCATENATED,
                rslice![ $( $($lifetime_indices),* )? ],
            );
        
        struct __ACPromoted<T>(T);

        impl<$($impl_gen)*> __ACPromoted<$type> 
        where $($($where_clause)*)?
        {
            $( const CONST_PARAM_UNERASED: &'static $const_ty = &$constants; )?
            const CONST_PARAM: &'static [$crate::abi_stability::ConstGeneric] = {
                &[
                    $(ignoring!(
                        ($constants)
                        $crate::abi_stability::ConstGeneric::new(
                            Self::CONST_PARAM_UNERASED,
                            $crate::abi_stability::ConstGenericVTableFor::NEW,
                        )
                    ),)?
                ]
            };

            const SHARED_VARS: &'static $crate::type_layout::SharedVars = {
                #[allow(unused_imports)]
                use $crate::abi_stability::stable_abi_trait::GetTypeLayoutCtor;

                &$crate::type_layout::SharedVars::new(
                    $mono_shared_vars,
                    rslice![ 
                        $( $( GetTypeLayoutCtor::<$ty_layout>::STABLE_ABI,)* )? 
                        $( $( GetTypeLayoutCtor::<$prefix_ty_layout>::PREFIX_STABLE_ABI,)* )? 
                    ],
                    $crate::std_types::RSlice::from_slice(Self::CONST_PARAM),
                )
            };
        }

        let $shared_vars=__ACPromoted::<Self>::SHARED_VARS;
    }
}


///////////////////////////////////////////////////////////////////////////////


/// Allows declaring a [`StaticRef`] constant.
/// 
/// This macro is for declaring associated constant of non-`'static` types.
///
/// # Example
///
/// This example demonstrates how you can construct a pointer to a vtable,
/// constructed at compile-time.
///
/// ```rust
/// use abi_stable::{
///     StableAbi,
///     extern_fn_panic_handling,
///     staticref,
///     pointer_trait::CallReferentDrop,
///     prefix_type::{PrefixTypeTrait, WithMetadata},
/// };
/// 
/// use std::{
///     mem::ManuallyDrop,
///     ops::Deref,
/// };
/// 
/// fn main(){
///     let boxed = BoxLike::new(100);
///    
///     assert_eq!(*boxed, 100);
///     assert_eq!(boxed.into_inner(), 100);
/// }
/// 
/// /// An ffi-safe `Box<T>`
/// #[repr(C)]
/// #[derive(StableAbi)]
/// pub struct BoxLike<T> {
///     data: *mut T,
///     
///     vtable: VTable_Ref<T>,
/// 
///     _marker: std::marker::PhantomData<T>,
/// }
/// 
/// impl<T> BoxLike<T>{
///     pub fn new(value: T) -> Self {
///         let box_ = Box::new(value);
///         
///         Self{
///             data: Box::into_raw(box_),
///             vtable: VTable::VTABLE,
///             _marker: std::marker::PhantomData,
///         }
///     }
/// 
///     /// Extracts the value this owns.
///     pub fn into_inner(self) -> T{
///         let this = ManuallyDrop::new(self);
///         unsafe{
///             // Must copy this before calling `self.vtable.destructor()`
///             // because otherwise it would be reading from a dangling pointer.
///             let ret = this.data.read();
///             this.vtable.destructor()(this.data,CallReferentDrop::No);
///             ret
///         }
///     }
/// }
/// 
/// 
/// impl<T> Drop for BoxLike<T>{
///     fn drop(&mut self){
///         unsafe{
///             self.vtable.destructor()(self.data, CallReferentDrop::Yes)
///         }
///     }
/// }
/// 
/// // `#[sabi(kind(Prefix))]` Declares this type as being a prefix-type,
/// // generating both of these types:
/// // 
/// //     - VTable_Prefix`: A struct with the fields up to (and including) the field with the 
/// //     `#[sabi(last_prefix_field)]` attribute.
/// // 
/// //     - VTable_Ref`: An ffi-safe pointer to `VTable`,with methods to get `VTable`'s fields.
/// // 
/// #[repr(C)]
/// #[derive(StableAbi)]
/// #[sabi(kind(Prefix))]
/// struct VTable<T>{
///     #[sabi(last_prefix_field)]
///     destructor: unsafe extern "C" fn(*mut T, CallReferentDrop),
/// }
/// 
/// impl<T> VTable<T>{
///    staticref!(const VTABLE_VAL: WithMetadata<Self> = WithMetadata::new(
///        PrefixTypeTrait::METADATA,
///        Self{
///            destructor: destroy_box::<T>,
///        },
///    ));
/// 
///    const VTABLE: VTable_Ref<T> = {
///        VTable_Ref( Self::VTABLE_VAL.as_prefix() )
///    };
/// } 
/// 
/// unsafe extern "C" fn destroy_box<T>(v: *mut T, call_drop: CallReferentDrop) {
///     extern_fn_panic_handling! {
///         let mut box_ = Box::from_raw(v as *mut ManuallyDrop<T>);
///         if call_drop == CallReferentDrop::Yes {
///             ManuallyDrop::drop(&mut *box_);
///         }
///         drop(box_);
///     }
/// }
/// 
/// 
/// impl<T> Deref for BoxLike<T> {
///     type Target=T;
/// 
///     fn deref(&self)->&T{
///         unsafe{
///             &(*self.data)
///         }
///     }
/// }
/// ```
///
/// [`StaticRef`]: ./sabi_types/struct.StaticRef.html
#[macro_export]
macro_rules! staticref{
    (
        $(
            $(#[$attr:meta])* $vis:vis const $name:ident : $ty:ty = $expr:expr
        );*
        $(;)?
    )=>{
        $(
            $crate::pmr::paste!{
                #[doc(hidden)]
                const [<__STATICREF_INIT_ $name>]: $ty = $expr;
             
                $(#[$attr])* 
                $vis const $name : $crate::sabi_types::StaticRef<$ty> = unsafe{
                    $crate::sabi_types::StaticRef::from_raw(&Self::[<__STATICREF_INIT_ $name>])
                };
            }
        )*
    };
}

///////////////////////////////////////////////////////////////////////////////


macro_rules! ignoring {
    (($($ignore:tt)*) $($passed:tt)* ) => {$($passed)*};
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


