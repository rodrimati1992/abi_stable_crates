
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
    type_layout::GenericParams
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
    type_layout::GenericParams
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
    ( $($lt:lifetime),*  ; $($ty:ty),* $(,)*  ; $($const_p:expr),*  ) => ({
        #[allow(unused_imports)]
        use $crate::{
            abi_stability::stable_abi_trait::SharedStableAbi,
            type_layout::GenericParams,
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


/// Constructs a `ReturnValueEquality<UTypeId>` that returns the UTypeId of the `$ty` type.
macro_rules! make_rve_utypeid {
    ($ty:ty) => (
        $crate::sabi_types::ReturnValueEquality{
            function:$crate::std_types::utypeid::new_utypeid::<$ty>
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
                sabi_types::{ReturnValueEquality},
                std_types::{StaticStr,utypeid::some_utypeid},
            };

            &TypeInfo{
                size:mem::size_of::<Self>(),
                alignment:mem::align_of::<Self>(),
                _uid:ReturnValueEquality{
                    function:some_utypeid::<Self>
                },
                name:StaticStr::new(stringify!($type)),
                module:StaticStr::new(module_path!()),
                package:StaticStr::new(env!("CARGO_PKG_NAME")),
                package_version:$crate::package_version_strings!(),
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
        
        Tag::arr(&[
            $( FromLiteral($elem).to_tag(), )*
        ])
    }};
    ({ $( $key:expr=>$value:expr ),* $(,)? })=>{{
        use $crate::type_layout::tagging::{FromLiteral,Tag};

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
        use $crate::type_layout::tagging::{Tag,FromLiteral};

        Tag::set(&[
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
            $crate::type_layout::ModPath::inside(module_path!()),
        )
    )
}


///////////////////////////////////////////////////////////////////////////////



/**
Constructs an `RVec<_>` using the same syntax that the `std::vec` macro uses.

# Example

```

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


