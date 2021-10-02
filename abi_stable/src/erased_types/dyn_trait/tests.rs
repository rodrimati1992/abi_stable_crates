use super::*;

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    collections::hash_map::DefaultHasher,
    fmt::{self, Display},
    hash::{Hash, Hasher},
};

use serde::Serialize;

use serde_json;

#[allow(unused_imports)]
use crate::{
    erased_types::{DynTrait, ImplType, InterfaceType, IteratorItem},
    impl_get_type_info,
    std_types::{RArc, RBox, RBoxError, RCow, RNone, ROption, RSome, RStr, RString},
    traits::IntoReprC,
    type_level::bools::{False, True},
    StableAbi,
};

#[allow(unused_imports)]
use core_extensions::SelfOps;

/// It doesn't need to be `#[repr(C)]` because  DynTrait puts it behind a pointer,
/// and is only interacted with through regular Rust functions.
#[derive(Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
struct Foo<T> {
    l: u32,
    r: u32,
    name: T,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Clone, Default, Display, Debug, Serialize, Deserialize, Ord, Hash))]
struct FooInterface;

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
    T: 'static,
{
    type Interface = FooInterface;
    const INFO: &'static crate::erased_types::TypeInfo = impl_get_type_info! { Foo<T> };
}

impl<'s, T> SerializeImplType<'s> for Foo<T>
where
    T: Serialize,
{
    type Interface = FooInterface;

    fn serialize_impl(&'s self) -> Result<RString, RBoxError> {
        match serde_json::to_string(self) {
            Ok(v) => Ok(v.into_c()),
            Err(e) => Err(RBoxError::new(e)),
        }
    }
}

impl<'s> SerializeProxyType<'s> for FooInterface {
    type Proxy = RString;
}

impl<'a> DeserializeDyn<'a, VirtualFoo<'static>> for FooInterface {
    type Proxy = RString;

    fn deserialize_dyn(s: RString) -> Result<VirtualFoo<'static>, RBoxError> {
        match ::serde_json::from_str::<Foo<String>>(&*s) {
            Ok(x) => Ok(DynTrait::from_value(x)),
            Err(e) => Err(RBoxError::new(e)),
        }
    }
}

type VirtualFoo<'a> = DynTrait<'a, RBox<()>, FooInterface>;

/////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType(Send, Sync, Debug))]
struct DebugInterface;

/////////////////////////////////

fn new_foo() -> Foo<String> {
    Foo {
        l: 1000,
        r: 100,
        name: "hello_world".into(),
    }
}

fn new_wrapped() -> VirtualFoo<'static> {
    DynTrait::from_value(new_foo())
}

#[test]
fn clone_test() {
    let wrapped_expected = Foo::<String>::default().piped(DynTrait::from_value);
    let wrapped = new_wrapped();

    {
        let cloned = wrapped.clone();

        assert_eq!(wrapped, cloned);
        assert_ne!(wrapped, wrapped_expected);
    }

    {
        let reborrow = wrapped.reborrow();
        let cloned = reborrow.clone();
        assert_eq!(reborrow, cloned);
        assert_ne!(wrapped, wrapped_expected);
    }
}

#[test]
fn default_test() {
    let concrete = Foo::<String>::default();
    let wrapped = new_wrapped().default();
    let wrapped_expected = Foo::<String>::default().piped(DynTrait::from_value);

    {
        assert_eq!(wrapped, wrapped_expected);
        assert_eq!(
            wrapped.downcast_as_impltype::<Foo<String>>().unwrap(),
            &concrete
        );
        assert_ne!(wrapped, new_wrapped());
    }

    {
        let reborrow = wrapped.reborrow();

        assert_eq!(reborrow, wrapped_expected);

        // This should not compile!!!!!
        // assert_eq!(reborrow.default(),wrapped_expected.reborrow());

        assert_eq!(
            reborrow.downcast_as_impltype::<Foo<String>>().unwrap(),
            &concrete
        );
        assert_ne!(reborrow, new_wrapped());
    }
}

#[test]
fn fmt_test() {
    let concrete = new_foo();
    let mut wrapped = new_wrapped();

    macro_rules! debug_test {
        ( $wrapped:ident ) => {{
            assert_eq!(format!("{:?}", concrete), format!("{:?}", $wrapped),);

            assert_eq!(format!("{:#?}", concrete), format!("{:#?}", $wrapped),);

            assert_eq!(format!("{}", concrete), format!("{}", $wrapped),);

            assert_eq!(format!("{:#}", concrete), format!("{:#}", $wrapped),);
        }};
    }

    debug_test!(wrapped);
    {
        let reborrow = wrapped.reborrow();
        debug_test!(reborrow);
    }
    {
        let reborrow = wrapped.reborrow_mut();
        debug_test!(reborrow);
    }
}

pub const JSON_0: &'static str = r#"
    {   
        "l":1000,
        "r":10,
        "name":"what the hell"
    }
"#;

#[test]
fn deserialize_test() {
    let json = JSON_0;

    let json_ss = serde_json::to_string(json).unwrap();

    let concrete = serde_json::from_str::<Foo<String>>(json).unwrap();

    let wrapped = VirtualFoo::deserialize_from_proxy(json.into()).unwrap();
    let wrapped = wrapped.reborrow();
    let wrapped2 = serde_json::from_str::<VirtualFoo<'static>>(&json_ss).unwrap();

    assert_eq!(
        serde_json::from_str::<VirtualFoo<'static>>(json).map_err(drop),
        Err(()),
    );

    assert_eq!(
        wrapped.downcast_as_impltype::<Foo<String>>().unwrap(),
        &concrete,
    );

    assert_eq!(
        wrapped2.downcast_as_impltype::<Foo<String>>().unwrap(),
        &concrete
    );
}

// Unfortunately: miri doesn't like calling `extern fn(*const ErasedType)` that
// were transmuted from `extern fn(*const ErasedType<T>)`
#[test]
fn serialize_test() {
    let concrete = new_foo();
    let mut wrapped = new_wrapped();

    macro_rules! serialize_test {
        ( $wrapped:ident ) => {{
            assert_eq!(
                &*concrete.piped_ref(serde_json::to_string).unwrap(),
                &*$wrapped.serialize_into_proxy().unwrap()
            );

            assert_eq!(
                concrete
                    .piped_ref(serde_json::to_string)
                    .unwrap()
                    .piped_ref(serde_json::to_string)
                    .unwrap(),
                $wrapped.piped_ref(serde_json::to_string).unwrap()
            );

            assert_eq!(
                $wrapped
                    .serialize_into_proxy()
                    .unwrap()
                    .piped_ref(serde_json::to_string)
                    .unwrap(),
                $wrapped.piped_ref(serde_json::to_string).unwrap()
            );
        }};
    }

    serialize_test!(wrapped);

    {
        let reborrow = wrapped.reborrow();
        serialize_test!(reborrow);
    }
    {
        let reborrow = wrapped.reborrow_mut();
        serialize_test!(reborrow);
    }
}

#[test]
fn cmp_test() {
    macro_rules! cmp_test {
        (
            wrapped_0=$wrapped_0:ident,
            wrapped_1=$wrapped_1:ident,
            wrapped_2=$wrapped_2:ident,
        ) => {{
            assert_eq!($wrapped_1 == $wrapped_0, false);
            assert_eq!($wrapped_1 <= $wrapped_0, false);
            assert_eq!($wrapped_1 >= $wrapped_0, true);
            assert_eq!($wrapped_1 < $wrapped_0, false);
            assert_eq!($wrapped_1 > $wrapped_0, true);
            assert_eq!($wrapped_1 != $wrapped_0, true);
            assert_eq!($wrapped_1.partial_cmp(&$wrapped_0), Some(Ordering::Greater));
            assert_eq!($wrapped_1.cmp(&$wrapped_0), Ordering::Greater);
            assert_eq!($wrapped_1.eq(&$wrapped_0), false);
            assert_eq!($wrapped_1.ne(&$wrapped_0), true);

            assert_eq!($wrapped_1 == $wrapped_1, true);
            assert_eq!($wrapped_1 <= $wrapped_1, true);
            assert_eq!($wrapped_1 >= $wrapped_1, true);
            assert_eq!($wrapped_1 < $wrapped_1, false);
            assert_eq!($wrapped_1 > $wrapped_1, false);
            assert_eq!($wrapped_1 != $wrapped_1, false);
            assert_eq!($wrapped_1.partial_cmp(&$wrapped_1), Some(Ordering::Equal));
            assert_eq!($wrapped_1.cmp(&$wrapped_1), Ordering::Equal);
            assert_eq!($wrapped_1.eq(&$wrapped_1), true);
            assert_eq!($wrapped_1.ne(&$wrapped_1), false);

            assert_eq!($wrapped_1 == $wrapped_2, false);
            assert_eq!($wrapped_1 <= $wrapped_2, true);
            assert_eq!($wrapped_1 >= $wrapped_2, false);
            assert_eq!($wrapped_1 < $wrapped_2, true);
            assert_eq!($wrapped_1 > $wrapped_2, false);
            assert_eq!($wrapped_1 != $wrapped_2, true);
            assert_eq!($wrapped_1.partial_cmp(&$wrapped_2), Some(Ordering::Less));
            assert_eq!($wrapped_1.cmp(&$wrapped_2), Ordering::Less);
            assert_eq!($wrapped_1.eq(&$wrapped_2), false);
            assert_eq!($wrapped_1.ne(&$wrapped_2), true);
        }};
    }

    let mut wrapped_0 = new_foo()
        .mutated(|x| x.l -= 100)
        .piped(DynTrait::from_value);
    let mut wrapped_1 = new_wrapped();
    let mut wrapped_2 = new_foo()
        .mutated(|x| x.l += 100)
        .piped(DynTrait::from_value);

    cmp_test! {
        wrapped_0=wrapped_0,
        wrapped_1=wrapped_1,
        wrapped_2=wrapped_2,
    }

    {
        let reborrow_0 = wrapped_0.reborrow();
        let reborrow_1 = wrapped_1.reborrow();
        let reborrow_2 = wrapped_2.reborrow();

        cmp_test! {
            wrapped_0=reborrow_0,
            wrapped_1=reborrow_1,
            wrapped_2=reborrow_2,
        }
    }
    {
        let reborrow_0 = wrapped_0.reborrow_mut();
        let reborrow_1 = wrapped_1.reborrow_mut();
        let reborrow_2 = wrapped_2.reborrow_mut();

        cmp_test! {
            wrapped_0=reborrow_0,
            wrapped_1=reborrow_1,
            wrapped_2=reborrow_2,
        }
    }
}

#[test]
fn hash_test() {
    fn hash_value<H: Hash>(v: &H) -> u64 {
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }

    {
        let mut wrapped = new_wrapped();
        assert_eq!(hash_value(&new_foo()), hash_value(&wrapped));

        {
            let reborrow = wrapped.reborrow();
            assert_eq!(hash_value(&new_foo()), hash_value(&reborrow));
        }
        {
            let reborrow_mut = wrapped.reborrow_mut();
            assert_eq!(hash_value(&new_foo()), hash_value(&reborrow_mut));
        }
    }

    {
        let concrete = Foo::<String>::default();
        let hash_concrete = hash_value(&concrete);
        let hash_wrapped = hash_value(&DynTrait::from_value(concrete.clone()));

        assert_eq!(hash_concrete, hash_wrapped);
    }
}

#[test]
fn from_any_test() {
    assert_eq!(
        DynTrait::from_value(new_foo()),
        DynTrait::from_any_value(new_foo(), FooInterface),
    );

    assert_eq!(
        DynTrait::from_ptr(RArc::new(new_foo())),
        DynTrait::from_any_ptr(RArc::new(new_foo()), FooInterface),
    );
}

#[test]
fn to_any_test() {
    let mut wrapped = DynTrait::from_any_value(new_foo(), FooInterface);

    macro_rules! to_unerased {
        ( $wrapped:expr ; $method:ident ; $expected:expr ) => {
            assert_eq!($wrapped.$method::<Foo<RString>>().map_err(drop), Err(()));

            assert_eq!($wrapped.$method::<Foo<String>>().unwrap(), $expected);
        };
    }

    to_unerased!( wrapped.clone() ; downcast_into_impltype     ; RBox::new(new_foo()) );
    to_unerased!( wrapped.clone() ; downcast_into ; RBox::new(new_foo()) );

    to_unerased!( wrapped ; downcast_as_impltype     ; &new_foo() );
    to_unerased!( wrapped ; downcast_as ; &new_foo() );

    to_unerased!( wrapped ; downcast_as_mut_impltype ; &mut new_foo() );
    to_unerased!( wrapped ; downcast_as_mut ; &mut new_foo() );

    {
        to_unerased!(wrapped.reborrow_mut(); downcast_into_impltype; RMut::new(&mut new_foo()));
        to_unerased!(wrapped.reborrow_mut() ; downcast_into ; RMut::new(&mut new_foo()));

        to_unerased!( wrapped.reborrow_mut() ; downcast_as_impltype     ; &new_foo() );
        to_unerased!( wrapped.reborrow_mut() ; downcast_as ; &new_foo() );

        to_unerased!( wrapped.reborrow_mut() ; downcast_as_mut_impltype ; &mut new_foo() );
        to_unerased!( wrapped.reborrow_mut() ; downcast_as_mut ; &mut new_foo() );
    }
    {
        to_unerased!( wrapped.reborrow() ; downcast_into_impltype; RRef::new(&new_foo()) );
        to_unerased!( wrapped.reborrow() ; downcast_into ; RRef::new(&new_foo()) );

        to_unerased!( wrapped.reborrow() ; downcast_as_impltype     ; &new_foo() );
        to_unerased!( wrapped.reborrow() ; downcast_as ; &new_foo() );
    }
}

//////////////////////////////////////////////////////////////////////

mod borrowing {
    use super::*;

    /// It doesn't need to be `#[repr(C)]` because  DynTrait puts it behind a pointer,
    /// and is only interacted with through regular Rust functions.
    #[derive(
        Default, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize,
    )]
    struct Foo<'a> {
        l: u32,
        r: u32,
        name: &'a str,
    }

    impl<'a> Foo<'a> {
        pub fn new(name: &'a str) -> Self {
            Self { l: 0, r: 0, name }
        }
    }

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType(
        Send,
        Sync,
        Clone,
        Default,
        Display,
        Debug,
        Serialize,
        Deserialize,
        Hash
    ))]
    struct FooInterface;

    impl<'s> SerializeProxyType<'s> for FooInterface {
        type Proxy = RString;
    }

    impl<'a> Display for Foo<'a> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "l:{}  r:{}  name:'", self.l, self.r)?;
            Display::fmt(&self.name, f)?;
            writeln!(f, "'")?;
            Ok(())
        }
    }

    impl ImplType for Foo<'static> {
        type Interface = FooInterface;
        const INFO: &'static crate::erased_types::TypeInfo = impl_get_type_info! { Foo<'static> };
    }

    impl<'a, 's> SerializeImplType<'s> for Foo<'a> {
        type Interface = FooInterface;

        fn serialize_impl(&self) -> Result<RString, RBoxError> {
            match serde_json::to_string(self) {
                Ok(v) => Ok(v.into_c()),
                Err(e) => Err(RBoxError::new(e)),
            }
        }
    }

    impl<'borr> DeserializeDyn<'borr, VirtualFoo<'borr>> for FooInterface {
        type Proxy = RStr<'borr>;

        fn deserialize_dyn(s: RStr<'borr>) -> Result<VirtualFoo<'borr>, RBoxError> {
            match ::serde_json::from_str::<Foo<'borr>>(s.as_str()) {
                Ok(x) => Ok(DynTrait::from_borrowing_value(x, FooInterface)),
                Err(e) => Err(RBoxError::new(e)),
            }
        }
    }

    type VirtualFoo<'a> = DynTrait<'a, RBox<()>, FooInterface>;

    fn check_fmt<'a>(foo: &Foo<'a>, wrapped: &VirtualFoo<'a>) {
        assert_eq!(format!("{:?}", wrapped), format!("{:?}", foo));
        assert_eq!(format!("{:#?}", wrapped), format!("{:#?}", foo));

        assert_eq!(format!("{}", wrapped), format!("{}", foo));
        assert_eq!(format!("{:#}", wrapped), format!("{:#}", foo));
    }

    #[test]
    fn cloning() {
        let name = "hello".to_string();
        let foo: Foo<'_> = Foo::new(&name);
        let wrapped = DynTrait::from_borrowing_value(foo.clone(), FooInterface);

        let cloned = wrapped.clone();

        check_fmt(&foo, &cloned);
    }

    #[test]
    fn default() {
        let name = "hello".to_string();
        let foo: Foo<'_> = Foo::new(&name);

        let default_name = "".to_string();
        let default_foo = Foo::new(&default_name);
        assert_eq!(default_foo, Default::default());

        let wrapped = DynTrait::from_borrowing_value(foo.clone(), FooInterface);

        let default_wrapped = wrapped.default();

        check_fmt(&default_foo, &default_wrapped);
    }

    #[test]
    fn formatting() {
        let name = "hello".to_string();
        let foo: Foo<'_> = Foo::new(&name);
        let wrapped = DynTrait::from_borrowing_value(foo.clone(), FooInterface);

        check_fmt(&foo, &wrapped);
    }

    #[test]
    fn serialize() {
        let name = "hello".to_string();
        let foo: Foo<'_> = Foo::new(&name);
        let wrapped = DynTrait::from_borrowing_value(foo.clone(), FooInterface);

        assert_eq!(
            &*serde_json::to_string(&foo).unwrap(),
            &*wrapped.serialize_into_proxy().unwrap(),
        );
    }

    #[test]
    fn deserialize() {
        let list: Vec<String> = vec![JSON_0.to_string()];

        for str_ in list.iter().map(|s| s.as_str()) {
            let foo: Foo<'_> = serde_json::from_str(str_).unwrap();
            let wrapped = VirtualFoo::deserialize_from_proxy(str_.into()).unwrap();

            check_fmt(&foo, &wrapped);
        }
    }

    ////////////////

    #[test]
    fn hash() {
        let name = "hello".to_string();
        let foo: Foo<'_> = Foo::new(&name);
        let wrapped = DynTrait::from_borrowing_value(foo.clone(), FooInterface);

        assert_eq!(HashedBytes::new(&foo), HashedBytes::new(&wrapped),);
    }

    #[derive(Debug, Default, PartialEq)]
    pub struct HashedBytes {
        bytes: Vec<u8>,
    }

    impl HashedBytes {
        pub fn new<T>(value: &T) -> Self
        where
            T: Hash,
        {
            let mut this = Self { bytes: Vec::new() };

            value.hash(&mut this);

            this
        }

        // pub fn bytes(&self)->&[u8]{
        //     &self.bytes
        // }
    }

    impl Hasher for HashedBytes {
        fn write(&mut self, bytes: &[u8]) {
            self.bytes.extend_from_slice(bytes);
        }

        fn finish(&self) -> u64 {
            // I'm not gonna call this
            0
        }
    }

    ////////////////

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType(Send, Sync, DoubleEndedIterator))]
    struct IterInterface;

    impl<'a> IteratorItem<'a> for IterInterface {
        type Item = &'a str;
    }

    fn iterator_from_lines<'borr>(s: &'borr str) -> DynTrait<'borr, RBox<()>, IterInterface> {
        let list = s.lines().collect::<Vec<&'borr str>>();
        DynTrait::from_borrowing_value(list.into_iter(), IterInterface)
    }

    fn exact_size_hint(n: usize) -> (usize, Option<usize>) {
        (n, Some(n))
    }

    #[test]
    fn iterator_collect() {
        let s = "line0\nline1\nline2".to_string();

        let actual = iterator_from_lines(&s).collect::<Vec<&str>>();

        let expected = vec!["line0", "line1", "line2"];

        assert_eq!(actual, expected);
    }

    #[test]
    fn iterator_next() {
        let s = "line0\nline1\nline2".to_string();
        let mut iter = iterator_from_lines(&s);

        assert_eq!(iter.size_hint(), exact_size_hint(3));
        assert_eq!(iter.next(), Some("line0"));

        assert_eq!(iter.size_hint(), exact_size_hint(2));
        assert_eq!(iter.next(), Some("line1"));

        assert_eq!(iter.size_hint(), exact_size_hint(1));
        assert_eq!(iter.next(), Some("line2"));

        assert_eq!(iter.size_hint(), exact_size_hint(0));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), exact_size_hint(0));
    }

    #[test]
    fn iterator_nth() {
        let s = "line0\nline1\nline2".to_string();

        assert_eq!(iterator_from_lines(&s).nth(0), Some("line0"));
        assert_eq!(iterator_from_lines(&s).nth(1), Some("line1"));
        assert_eq!(iterator_from_lines(&s).nth(2), Some("line2"));
        assert_eq!(iterator_from_lines(&s).nth(3), None);
    }

    #[test]
    fn iterator_count() {
        let s = "line0\nline1\nline2".to_string();

        assert_eq!(iterator_from_lines(&s).count(), 3);
        assert_eq!(iterator_from_lines(&s).skip(0).count(), 3);
        assert_eq!(iterator_from_lines(&s).skip(1).count(), 2);
        assert_eq!(iterator_from_lines(&s).skip(2).count(), 1);
        assert_eq!(iterator_from_lines(&s).skip(3).count(), 0);
        assert_eq!(iterator_from_lines(&s).skip(4).count(), 0);
    }

    #[test]
    fn iterator_last() {
        let s0 = "line0".to_string();
        let s1 = "line0\nline1".to_string();
        let s2 = "line0\nline1\nline2".to_string();

        assert_eq!(iterator_from_lines(&s0).last(), Some("line0"));
        assert_eq!(iterator_from_lines(&s1).last(), Some("line1"));
        assert_eq!(iterator_from_lines(&s2).last(), Some("line2"));
    }

    #[test]
    fn iterator_skip_eager() {
        let s = "line0\nline1\nline2".to_string();

        let skipping = |how_many: usize| {
            let mut iter = iterator_from_lines(&s);
            iter.skip_eager(how_many);
            iter
        };

        assert_eq!(skipping(0).next(), Some("line0"));
        assert_eq!(skipping(0).count(), 3);
        assert_eq!(skipping(1).next(), Some("line1"));
        assert_eq!(skipping(1).count(), 2);
        assert_eq!(skipping(2).next(), Some("line2"));
        assert_eq!(skipping(2).count(), 1);
        assert_eq!(skipping(3).next(), None);
        assert_eq!(skipping(3).count(), 0);
    }

    #[test]
    fn iterator_extending_rvec() {
        let s = "line0\nline1\nline2".to_string();

        let collected =
            |how_many: Option<usize>| s.lines().take(how_many.unwrap_or(!0)).collect::<RVec<_>>();

        let extending = |how_many: ROption<usize>| {
            let mut iter = iterator_from_lines(&s);
            let mut buffer = RVec::new();
            iter.extending_rvec(&mut buffer, how_many);
            buffer
        };

        assert_eq!(extending(RNone), collected(None));
        assert_eq!(extending(RSome(0)), collected(Some(0)));
        assert_eq!(extending(RSome(1)), collected(Some(1)));
        assert_eq!(extending(RSome(2)), collected(Some(2)));
        assert_eq!(extending(RSome(3)), collected(Some(3)));
    }

    ////////////////

    #[test]
    fn iterator_next_back() {
        let s = "line0\nline1\nline2".to_string();
        let mut iter = iterator_from_lines(&s);

        assert_eq!(iter.size_hint(), exact_size_hint(3));
        assert_eq!(iter.next_back(), Some("line2"));

        assert_eq!(iter.size_hint(), exact_size_hint(2));
        assert_eq!(iter.next_back(), Some("line1"));

        assert_eq!(iter.size_hint(), exact_size_hint(1));
        assert_eq!(iter.next_back(), Some("line0"));

        assert_eq!(iter.size_hint(), exact_size_hint(0));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.size_hint(), exact_size_hint(0));
    }

    #[test]
    fn iterator_nth_back() {
        let s = "line0\nline1\nline2".to_string();

        assert_eq!(iterator_from_lines(&s).nth_back_(0), Some("line2"));
        assert_eq!(iterator_from_lines(&s).nth_back_(1), Some("line1"));
        assert_eq!(iterator_from_lines(&s).nth_back_(2), Some("line0"));
        assert_eq!(iterator_from_lines(&s).nth_back_(3), None);
    }
    #[test]
    fn iterator_extending_rvec_back() {
        let s = "line0\nline1\nline2".to_string();

        let collected = |how_many: Option<usize>| {
            s.lines()
                .rev()
                .take(how_many.unwrap_or(!0))
                .collect::<RVec<_>>()
        };

        let extending = |how_many: ROption<usize>| {
            let mut iter = iterator_from_lines(&s);
            let mut buffer = RVec::new();
            iter.extending_rvec_back(&mut buffer, how_many);
            buffer
        };

        assert_eq!(extending(RNone), collected(None));
        assert_eq!(extending(RSome(0)), collected(Some(0)));
        assert_eq!(extending(RSome(1)), collected(Some(1)));
        assert_eq!(extending(RSome(2)), collected(Some(2)));
        assert_eq!(extending(RSome(3)), collected(Some(3)));
    }

    ////////////////

    #[test]
    fn is_same_type() {
        let value: String = "hello".to_string();

        let wrapped = DynTrait::from_borrowing_value(value.clone(), ());
        let wrapped = wrapped.reborrow();

        // Creating a DynTrait with a different interface so that it
        // creates a different vtable.
        let dbg_wrapped = DynTrait::from_borrowing_value(value.clone(), DebugInterface);

        assert!(!wrapped.sabi_is_same_type(&dbg_wrapped));
    }

    #[test]
    fn unerase_should_not_work() {
        let value: String = "hello".to_string();

        macro_rules! to_unerased {
            ( $wrapped:expr ; $( $method:ident ),* $(,)* ) => (
                $(
                    assert_eq!(
                        $wrapped.$method ::<String>().map_err(drop),
                        Err(())
                    );
                )*
            )
        }

        to_unerased!(
            DynTrait::from_borrowing_value(value.clone(),());
            downcast_into,
        );

        to_unerased!(
            DynTrait::from_borrowing_value(value.clone(),());
            downcast_as,
            downcast_as_mut,
        );
    }

    ///////////////////////////////////////////////////////////////////////////////////

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType(Send, Sync, FmtWrite))]
    struct FmtInterface;

    #[test]
    fn fmt_write() {
        use std::fmt::Write;
        let mut s = String::new();
        {
            let mut wrapped = DynTrait::from_any_ptr(&mut s, FmtInterface);
            let mut wrapped = wrapped.reborrow_mut();
            wrapped.write_char('¿').unwrap();
            wrapped.write_str("Hello").unwrap();
            wrapped.write_char('?').unwrap();
        }
        assert_eq!(&*s, "¿Hello?");
    }

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType(Send, Sync, IoWrite, IoSeek, IoRead, IoBufRead))]
    struct IoInterface;

    #[test]
    fn io_write() {
        use std::io::{Cursor, Write};

        const FILLER: u8 = 255;

        let mut buff = vec![FILLER; 9];
        let mut buff = buff[..].piped_mut(Cursor::new);
        {
            let mut wrapped = DynTrait::from_borrowing_ptr(&mut buff, IoInterface);
            assert_eq!(wrapped.write(&[0, 1]).map_err(drop), Ok(2));

            wrapped.write_all(&[2, 3, 4, 5]).unwrap();
        }
        assert_eq!(
            &**buff.get_ref(),
            &[0, 1, 2, 3, 4, 5, FILLER, FILLER, FILLER][..]
        );
        {
            let mut wrapped = DynTrait::from_borrowing_ptr(&mut buff, IoInterface);

            wrapped
                .write_all(&[2, 3, 4, 5, 6, 7, 8, 9, 10])
                .unwrap_err();

            wrapped.flush().unwrap();
        }
    }

    #[test]
    fn io_read() {
        use std::io::{Cursor, Read};

        let mut buff = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10].piped(Cursor::new);
        let mut out = vec![0; 400];

        let mut wrapped = DynTrait::from_any_ptr(&mut buff, IoInterface);
        assert_eq!(wrapped.read(&mut out[..3]).map_err(drop), Ok(3));
        assert_eq!(&out[..3], &[1, 2, 3][..]);

        assert_eq!(wrapped.read_exact(&mut out[4..8]).map_err(drop), Ok(()));
        assert_eq!(&out[4..8], &[4, 5, 6, 7][..]);

        assert_eq!(wrapped.read_exact(&mut out[8..]).map_err(drop), Err(()));
    }

    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(impl_InterfaceType(Send, Sync, IoRead, IoBufRead))]
    struct IoBufReadInterface;

    #[test]
    fn io_bufread() {
        use std::io::{BufRead, Cursor};

        let s = "line0\nline1\nline2".as_bytes().piped(Cursor::new);

        let wrapped = DynTrait::from_borrowing_value(s, IoBufReadInterface);

        assert_eq!(
            wrapped.lines().collect::<Result<Vec<String>, _>>().unwrap(),
            vec![
                "line0".to_string(),
                "line1".to_string(),
                "line2".to_string(),
            ]
        );
    }

    #[test]
    fn io_seek() {
        use std::io::{Cursor, Read, Seek, SeekFrom};

        let mut buff = vec![255, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10].piped(Cursor::new);
        let mut out = vec![0; 400];

        let mut wrapped = DynTrait::from_any_ptr(&mut buff, IoInterface);

        {
            wrapped.seek(SeekFrom::Start(1)).unwrap();
            assert_eq!(wrapped.read_exact(&mut out[..4]).map_err(drop), Ok(()));
            assert_eq!(&out[..4], &[1, 2, 3, 4][..]);
        }
        {
            wrapped.seek(SeekFrom::End(-3)).unwrap();
            assert_eq!(wrapped.read_exact(&mut out[4..7]).map_err(drop), Ok(()));
            assert_eq!(&out[..7], &[1, 2, 3, 4, 8, 9, 10][..]);
        }
        {
            wrapped.seek(SeekFrom::Current(-4)).unwrap();
            assert_eq!(wrapped.read_exact(&mut out[7..8]).map_err(drop), Ok(()));
            assert_eq!(&out[..8], &[1, 2, 3, 4, 8, 9, 10, 7][..]);
        }
    }
}
