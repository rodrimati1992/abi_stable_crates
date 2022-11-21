use std::mem;

use crate::{
    sabi_trait::prelude::*,
    std_types::{RBox, RStr},
    type_level::bools::*,
    *,
};

use abi_stable_shared::test_utils::must_panic;

mod empty {
    use super::*;
    #[sabi_trait]
    pub trait Trait {}

    impl Trait for () {}

    impl Trait for True {}
}

mod method_no_default {
    use super::*;
    #[sabi_trait]
    pub trait Trait {
        fn apply(&self, l: u32, r: u32) -> u32;
    }

    impl Trait for () {
        fn apply(&self, l: u32, r: u32) -> u32 {
            (l + r) * 2
        }
    }
}

mod method_disabled_one_default {
    use super::*;

    #[sabi_trait]
    pub trait Trait {
        fn first_method(&self) -> u32 {
            0xF000
        }
        #[sabi(no_default_fallback)]
        fn apply(&self, l: u32, r: u32) -> u32 {
            (l + r) * 3
        }
        fn last_method(&self) -> u32 {
            0xFAAA
        }
    }

    impl Trait for () {
        fn apply(&self, l: u32, r: u32) -> u32 {
            (l + r) * 4
        }
    }
}

mod method_disabled_all_default {
    use super::*;

    #[sabi_trait]
    #[sabi(no_default_fallback)]
    pub trait Trait {
        fn first_method(&self) -> u32 {
            0xF000
        }
        fn last_method(&self) -> u32 {
            0xFAAA
        }
    }

    impl Trait for () {}
}

mod method_default {
    use super::*;
    #[sabi_trait]
    pub trait Trait {
        fn apply(&self, l: u32, r: u32) -> u32 {
            (l + r) * 3
        }
    }

    impl Trait for () {}

    impl Trait for True {
        fn apply(&self, l: u32, r: u32) -> u32 {
            (l + r) * 4
        }
    }
}

#[test]
fn downcasting_tests() {
    unsafe {
        use self::method_disabled_one_default::*;
        let empty = empty::Trait_TO::from_value((), TD_Opaque);
        // these transmutes are for testing DynTraits created across library versions
        let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(empty);
        assert_eq!(object.first_method(), 0xF000);
        assert_eq!(object.last_method(), 0xFAAA);
        must_panic(|| object.apply(2, 5)).unwrap();
    }
    unsafe {
        use self::method_disabled_all_default::*;
        let empty = empty::Trait_TO::from_value((), TD_Opaque);
        let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(empty);
        must_panic(|| object.first_method()).unwrap();
        must_panic(|| object.last_method()).unwrap();
    }
    unsafe {
        use self::method_no_default::*;
        let empty = empty::Trait_TO::from_value((), TD_Opaque);
        let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(empty);
        must_panic(|| object.apply(2, 5)).unwrap();
    }
    unsafe {
        use self::method_default::*;
        let empty = empty::Trait_TO::from_value((), TD_Opaque);
        let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(empty);
        assert_eq!(object.apply(2, 5), 21);
    }

    {
        let no_default = method_no_default::Trait_TO::from_value((), TD_Opaque);
        {
            assert_eq!(no_default.apply(2, 5), 14);
        }
        unsafe {
            use self::method_default::*;
            let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(no_default);
            assert_eq!(object.apply(2, 5), 14);
        }
    }
    {
        let with_default = method_default::Trait_TO::from_value(True, TD_Opaque);
        {
            assert_eq!(with_default.apply(2, 5), 28);
        }
        unsafe {
            use self::method_no_default::*;
            let object = mem::transmute::<_, Trait_TO<'_, RBox<()>>>(with_default);
            assert_eq!(object.apply(2, 5), 28);
        }
    }
}

#[sabi_trait]
trait DefaultMethodPair {
    fn foo(&self, x: u32) -> u32 {
        self.bar(x + 10)
    }
    fn bar(&self, y: u32) -> u32 {
        self.baz(y + 20)
    }
    fn baz(&self, z: u32) -> u32 {
        z + 40
    }
}

struct A;
struct B;
struct C;

impl DefaultMethodPair for A {
    fn foo(&self, x: u32) -> u32 {
        x + 100
    }
}

impl DefaultMethodPair for B {
    fn bar(&self, y: u32) -> u32 {
        y + 200
    }
}

impl DefaultMethodPair for C {
    fn baz(&self, z: u32) -> u32 {
        z + 300
    }
}

#[test]
fn default_methods() {
    let a = DefaultMethodPair_TO::from_value(A, TD_Opaque);
    let b = DefaultMethodPair_TO::from_value(B, TD_Opaque);
    let c = DefaultMethodPair_TO::from_value(C, TD_Opaque);

    assert_eq!(a.foo(1), 101);
    assert_eq!(b.foo(1), 211);
    assert_eq!(c.foo(1), 331);
}

/*////////////////////////////////////////////////////////////////////////////////
Test that #[sabi(no_trait_impl)] disables the trait impl for the trait object.
*/////////////////////////////////////////////////////////////////////////////////

#[sabi_trait]
#[sabi(no_trait_impl)]
trait NoTraitImplA {}

impl<P> NoTraitImplA for NoTraitImplA_TO<'_, P> where P: crate::pointer_trait::GetPointerKind {}

#[sabi_trait]
#[sabi(no_trait_impl)]
trait NoTraitImplB {}

impl<This: ?Sized> NoTraitImplB for This {}

/*////////////////////////////////////////////////////////////////////////////////
Test that prefix methods can have a default impl.
*/////////////////////////////////////////////////////////////////////////////////

mod defaulted_prefix_method {
    use super::*;
    #[sabi_trait]
    pub trait Trait {
        #[sabi(last_prefix_field)]
        fn apply(&self) -> u32 {
            3
        }
    }

    impl Trait for () {}
    impl Trait for u32 {
        fn apply(&self) -> u32 {
            *self + 5
        }
    }
}

#[test]
fn defaulted_prefix_method_works() {
    use defaulted_prefix_method::Trait_TO;
    {
        let obj = Trait_TO::from_value((), TD_Opaque);
        assert_eq!(obj.apply(), 3);
    }
    {
        let obj = Trait_TO::from_value(0u32, TD_Opaque);
        assert_eq!(obj.apply(), 5);
    }
    {
        let obj = Trait_TO::from_value(10u32, TD_Opaque);
        assert_eq!(obj.apply(), 15);
    }
}

/*////////////////////////////////////////////////////////////////////////////////
Test all the kinds of borrows in return types.
*/////////////////////////////////////////////////////////////////////////////////

#[sabi_trait]
trait BorrowKinds {
    fn ref_borrow_a(&self) -> &bool {
        &true
    }
    fn ref_borrow_b(&self) -> &RStr<'_> {
        &rstr!("foo")
    }

    fn other_borrow(&self) -> RStr<'_> {
        RStr::from_str("bar")
    }

    fn non_self_borrow(&self) -> RStr<'static> {
        RStr::from_str("baz")
    }

    fn mut_borrow(&mut self) -> &mut u32;

    fn not_borrow(&self) -> u64 {
        89
    }
}

impl BorrowKinds for u32 {
    fn mut_borrow(&mut self) -> &mut u32 {
        self
    }
}

#[test]
fn borrow_kinds() {
    let mut obj = BorrowKinds_TO::from_value(3u32, TD_Opaque);

    assert_eq!(obj.ref_borrow_a(), &true);
    assert_eq!(obj.ref_borrow_b().as_str(), "foo");
    assert_eq!(obj.other_borrow().as_str(), "bar");
    assert_eq!(obj.non_self_borrow().as_str(), "baz");
    assert_eq!(*obj.mut_borrow(), 3);
    assert_eq!(obj.not_borrow(), 89);
}

////////////////////////////////////////////////////////////////////////////////

mod has_docs {
    /// above
    #[crate::sabi_trait]
    /// below
    #[sabi(debug_output_tokens)]
    pub trait HasDocs {
        /// above2
        /// below2
        fn foo(&self) {}
    }
}

fn remove_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[test]
fn docs_are_included_test() {
    let no_whitespace = remove_whitespace(has_docs::TOKENS);
    assert!(
        no_whitespace.contains(&*remove_whitespace(
            "
                #[doc=\"above\"]
                #[doc=\"below\"]
                pub trait HasDocs
            "
        )) && no_whitespace.contains(&*remove_whitespace(
            "
                #[doc=\"above2\"]
                #[doc=\"below2\"]
                fn foo(&self
            "
        )) && no_whitespace.contains(&*remove_whitespace("pub fn foo(&self")),
        "{}",
        has_docs::TOKENS
    );
}
