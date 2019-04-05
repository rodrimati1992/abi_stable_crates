use super::*;

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    collections::HashSet,
    fmt::{self, Debug, Display},
    marker::PhantomData,
};

use serde_json;

#[macro_use]
extern crate serde_derive;

#[allow(unused_imports)]
use crate::{
    abi_stability::{check_abi_stability, SharedStableAbi},
    erased_types::VirtualWrapper,
    impl_get_type_info,
    traits::{DeserializeImplType, False, ImplType, InterfaceType, SerializeImplType, True},
    IntoReprC, OpaqueType, RArc, RBox, RBoxError, RCow, RStr, RString, StableAbi, StaticStr,
};

use core_extensions::prelude::*;

/// It doesn't need to be `#[repr(C)]` because  VirtualWrapper puts it behind a pointer,
/// and is only interacted with through regular Rust functions.
#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
struct Foo<T> {
    l: u32,
    r: u32,
    name: T,
}

struct FooInterface;

impl_get_type_info! {
    impl[T:'static] GetTypeInfo for Foo[T]

    version=0,1,0;
}

impl<S> Display for Foo<S>
where
    S: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "l:{}  r:{}  name:'{}'", self.l, self.r, self.name,)
    }
}

impl<T> ImplType for Foo<T>
where
    T: 'static + Send + Sync,
{
    type Interface = FooInterface;
}

impl<T> SerializeImplType for Foo<T>
where
    T: 'static + Send + Sync + Debug,
{
    fn serialize_impl(&self) -> Result<RCow<'_, str>, RBoxError> {
        Ok(format!("{:#?}", self).into_c().piped(RCow::Owned))
    }
}

impl InterfaceType for FooInterface {
    type Clone = True;

    type Default = True;

    type Display = True;

    type Debug = True;

    type Serialize = True;

    type Deserialize = True;

    type Eq = True;

    type PartialEq = True;

    type Ord = True;

    type PartialOrd = True;

    type Hash = True;
}

impl DeserializeImplType for FooInterface {
    type Deserialized = VirtualFoo;

    fn deserialize_impl(s: RStr<'_>) -> Result<Self::Deserialized, RBoxError> {
        match ::serde_json::from_str::<Foo<RString>>(&*s) {
            Ok(x) => Ok(VirtualWrapper::from_value(x)),
            Err(e) => Err(RBoxError::new(e)),
        }
    }
}

type VirtualFoo = VirtualWrapper<RBox<OpaqueType<FooInterface>>>;

/////////////////////////////////

mod helloa {
    use super::*;
    #[derive(StableAbi)]
    #[repr(C)]
    #[sabi(kind(unsafe_Prefix))]
    pub struct Hello<'a, 'b> {
        _marker: PhantomData<(&'a (), &'b ())>,
        what: &'a (),
    }
}

#[allow(dead_code)]
mod hellob {
    use super::*;
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(unsafe_Prefix))]
    pub struct Hello<'a, 'b> {
        _marker: PhantomData<(&'a (), &'b ())>,
        what: &'a (),
        nope: u32,
    }
}

fn main() {
    for _ in 0..10 {
        use core_extensions::measure_time;
        use std::fmt::Write;

        println!("{}", StaticStr::new("hello world."));

        // type Ty0<T> = Option<&'static mut [T; 12]>;
        // type TyWhat<T> = Ty0<Ty0<Ty0<T>>>;

        // let ty_1 = <&helloa::Hello>::ABI_INFO;
        // let ty_2 = <&hellob::Hello>::ABI_INFO;
        // let ty_3 = <VirtualWrapper<RArc<OpaqueType<FooInterface>>>>::ABI_INFO;
        let ty_4 = <hellob::Hello>::ABI_INFO;

        let check = |l, r| {
            let (dur, res) = measure_time::measure(|| check_abi_stability(l, r));
            let mut buffer = String::new();
            match res {
                Ok(_) => writeln!(buffer, "no errors"),
                Err(e) => writeln!(buffer, "{}", e),
            }
            .drop_();
            writeln!(buffer, "time taken to compare:{}", dur).drop_();
            buffer
        };

        let separator = "\n--------------------------------\n";

        // println!("{S}{}", check(ty_1, ty_2), S = separator);
        // println!("{S}{}", check(ty_3, ty_3), S = separator);
        println!("{S}{}", check(ty_4, ty_4), S = separator);
        // println!("{S}{}", check(ty_2, ty_1), S = separator);
    }
}

/////////////////////////////////

fn _main() {
    let str_ = serde_json::to_string(
        r#"
        {   
            "l":1000,
            "r":10,
            "name":"what the hell"
        }
    "#,
    )
    .unwrap();

    let concrete = str_
        .piped_ref(|x| serde_json::from_str::<RString>(&x))
        .unwrap()
        .piped(|x| serde_json::from_str::<Foo<RString>>(&x))
        .unwrap();
    let n = serde_json::from_str::<VirtualFoo>(&str_).unwrap();

    {
        // Testing the Display impl.
        assert_eq!(concrete.to_string(), n.to_string());
        println!("{}", n);
    }

    {
        // Testing the Defaut impl
        assert_eq!(
            Foo::<RString>::default().piped(VirtualWrapper::from_value),
            n.default()
        );
    }

    println!("{:#?}", n);

    assert_eq!(n == n, true);
    assert_eq!(n <= n, true);
    assert_eq!(n >= n, true);
    assert_eq!(n < n, false);
    assert_eq!(n > n, false);
    assert_eq!(n != n, false);
    assert_eq!(n.partial_cmp(&n), Some(Ordering::Equal));
    assert_eq!(n.cmp(&n), Ordering::Equal);
    assert_eq!(n.eq(&n), true);
    assert_eq!(n.ne(&n), false);

    let mut clone = n.clone();
    {
        // Testing the Clone impl
        assert_ne!(n.object_address(), clone.object_address());
        assert_eq!(n, clone);
    }

    {
        // Testing that mutation of the concrete type affects the erased type.
        let unerased = clone.as_unerased_mut::<Foo<RString>>().unwrap();
        unerased.name.push_str("hello");
        println!("{:#?}", clone);
    }

    {
        // Testing as_unerased
        let res = clone.as_unerased::<Foo<u32>>();
        assert!(res.is_err(), "{:#?}", res);
    }

    {
        //Testing the consistency of Hash
        let mut map = HashSet::new();
        map.insert(n.clone());
        assert_eq!(map.len(), 1);
        map.insert(n.clone());
        assert_eq!(map.len(), 1);
        assert!(map.contains(&n), "{:#?}", map);
    }
}
