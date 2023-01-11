use super::*;

use crate::{
    inline_storage::alignment::{AlignTo1, AlignTo16, AlignTo2, AlignTo4, AlignTo8},
    nonexhaustive_enum::{
        examples::{
            command_a, command_b, command_c, command_h_mismatched_discriminant, command_serde,
            const_expr_size_align, generic_a, generic_b, many_ranges_a, many_ranges_b,
        },
        GetEnumInfo,
    },
    test_utils::{check_formatting_equivalence, must_panic},
};

use core_extensions::SelfOps;

use std::{
    cmp::{Ord, Ordering, PartialEq, PartialOrd},
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

#[test]
fn construct_deconstruct() {
    macro_rules! construct_deconstruct_cases {
        ($NE:ident :: $ctor:ident($($extra_args:tt)*)) => {
            {
                use self::command_a::Foo as FooA;
                let mut variant_a = $NE::$ctor(FooA::A, $($extra_args)*);
                let mut variant_b = $NE::$ctor(FooA::B(11), $($extra_args)*);

                assert_eq!(variant_a.as_enum(), Ok(&FooA::A));
                assert_eq!(variant_b.as_enum(), Ok(&FooA::B(11)));

                assert_eq!(variant_a.as_enum_mut(), Ok(&mut FooA::A));
                assert_eq!(variant_b.as_enum_mut(), Ok(&mut FooA::B(11)));

                assert_eq!(variant_a.into_enum(), Ok(FooA::A));
                assert_eq!(variant_b.into_enum(), Ok(FooA::B(11)));
            }
            {
                use self::command_b::Foo as FooB;

                let mut variant_a = $NE::$ctor(FooB::A, $($extra_args)*);
                let mut variant_b = $NE::$ctor(FooB::B(11), $($extra_args)*);
                let mut variant_c = $NE::$ctor(FooB::C, $($extra_args)*);

                assert_eq!(variant_a.as_enum(), Ok(&FooB::A));
                assert_eq!(variant_b.as_enum(), Ok(&FooB::B(11)));
                assert_eq!(variant_c.as_enum(), Ok(&FooB::C));

                assert_eq!(variant_a.as_enum_mut(), Ok(&mut FooB::A));
                assert_eq!(variant_b.as_enum_mut(), Ok(&mut FooB::B(11)));
                assert_eq!(variant_c.as_enum_mut(), Ok(&mut FooB::C));

                assert_eq!(variant_a.into_enum(), Ok(FooB::A));
                assert_eq!(variant_b.into_enum(), Ok(FooB::B(11)));
                assert_eq!(variant_c.into_enum(), Ok(FooB::C));
            }
        };
    }

    construct_deconstruct_cases! {NonExhaustive::new()}
    construct_deconstruct_cases! {NonExhaustiveFor::new()}
}

#[test]
fn construct_panic() {
    use self::generic_b::{Foo, Foo_Interface, Foo_Storage};

    type NE<E> = NonExhaustive<E, Foo_Storage, Foo_Interface>;

    macro_rules! passing_ctor {
        ($enum_ty:ty) => {{
            type ET = $enum_ty;
            let runtime = <NE<ET>>::with_storage_and_interface(ET::A);
            let const_ = <NE<ET>>::new(ET::B);

            assert_eq!(runtime, ET::A);
            assert_eq!(const_, ET::B);
        }};
    }

    macro_rules! failing_ctor {
        ($enum_ty:ty) => {{
            type ET = $enum_ty;
            must_panic(|| <NE<ET>>::with_storage_and_interface(ET::B)).unwrap();
            must_panic(|| <NE<ET>>::new(ET::A)).unwrap();
        }};
    }

    passing_ctor! {Foo<AlignTo8<[u8; 0]>>}
    passing_ctor! {Foo<AlignTo8<[u8; 56]>>}

    // too large
    failing_ctor! {Foo<AlignTo8<[u8; 64]>>}
    // too aligned
    failing_ctor! {Foo<AlignTo16<[u8; 0]>>}
    // too large, too aligned
    failing_ctor! {Foo<AlignTo16<[u8; 64]>>}
}

#[test]
fn const_expr_size_align_test() {
    use self::const_expr_size_align::{Foo, Foo_Interface, Foo_Storage};

    type NE<E> = NonExhaustive<E, Foo_Storage, Foo_Interface>;

    macro_rules! passing_ctor {
        ($enum_ty:ty) => {{
            type ET = $enum_ty;
            let const_ = <NE<ET>>::new(ET::B);
            assert_eq!(const_, ET::B);
        }};
    }

    macro_rules! failing_ctor {
        ($enum_ty:ty) => {{
            type ET = $enum_ty;
            must_panic(|| <NE<ET>>::new(ET::A)).unwrap();
        }};
    }

    passing_ctor! {Foo<AlignTo2<[u8; 0]>>}
    passing_ctor! {Foo<AlignTo1<[u8; 9]>>}
    passing_ctor! {Foo<AlignTo2<[u8; 8]>>}

    // too large
    failing_ctor! {Foo<AlignTo2<[u8; 9]>>}
    // too aligned
    failing_ctor! {Foo<AlignTo4<[u8; 0]>>}
    // too large, too aligned
    failing_ctor! {Foo<AlignTo4<[u8; 64]>>}
}

#[test]
fn get_discriminant() {
    {
        use self::command_c::Foo as FooC;
        let wrapped_a = NonExhaustive::new(FooC::A);
        let wrapped_b = NonExhaustive::new(FooC::B(11));
        let wrapped_c = NonExhaustive::new(FooC::C);
        let wrapped_d = NonExhaustive::new(FooC::D {
            name: "what".into(),
        });

        assert_eq!(wrapped_a.get_discriminant(), 0);
        assert_eq!(wrapped_b.get_discriminant(), 1);
        assert_eq!(wrapped_c.get_discriminant(), 2);
        assert_eq!(wrapped_d.get_discriminant(), 3);
    }
    {
        use self::command_h_mismatched_discriminant::Foo;
        let wrapped_a = NonExhaustive::new(Foo::A);
        let wrapped_b = NonExhaustive::new(Foo::B);
        let wrapped_c = NonExhaustive::new(Foo::C);

        assert_eq!(wrapped_a.get_discriminant(), 40);
        assert_eq!(wrapped_b.get_discriminant(), 41);
        assert_eq!(wrapped_c.get_discriminant(), 42);
    }
}

#[test]
fn is_valid_discriminant() {
    {
        use self::command_c::Foo as FooC;
        assert_eq!(FooC::is_valid_discriminant(0), true);
        assert_eq!(FooC::is_valid_discriminant(1), true);
        assert_eq!(FooC::is_valid_discriminant(2), true);
        assert_eq!(FooC::is_valid_discriminant(3), true);
        assert_eq!(FooC::is_valid_discriminant(4), false);
        assert_eq!(FooC::is_valid_discriminant(5), false);
    }
    {
        use self::command_h_mismatched_discriminant::Foo;
        assert_eq!(Foo::is_valid_discriminant(0), false);
        assert_eq!(Foo::is_valid_discriminant(39), false);
        assert_eq!(Foo::is_valid_discriminant(40), true);
        assert_eq!(Foo::is_valid_discriminant(41), true);
        assert_eq!(Foo::is_valid_discriminant(42), true);
        assert_eq!(Foo::is_valid_discriminant(43), false);
        assert_eq!(Foo::is_valid_discriminant(44), false);
    }
    {
        use self::many_ranges_a::Foo;
        assert_eq!(Foo::is_valid_discriminant(0), true);
        assert_eq!(Foo::is_valid_discriminant(1), false);
        assert_eq!(Foo::is_valid_discriminant(2), false);
        assert_eq!(Foo::is_valid_discriminant(39), false);
        assert_eq!(Foo::is_valid_discriminant(40), true);
        assert_eq!(Foo::is_valid_discriminant(41), true);
        assert_eq!(Foo::is_valid_discriminant(42), true);
        assert_eq!(Foo::is_valid_discriminant(43), false);
        assert_eq!(Foo::is_valid_discriminant(44), false);
        assert_eq!(Foo::is_valid_discriminant(58), false);
        assert_eq!(Foo::is_valid_discriminant(59), false);
        assert_eq!(Foo::is_valid_discriminant(60), true);
        assert_eq!(Foo::is_valid_discriminant(61), true);
        assert_eq!(Foo::is_valid_discriminant(62), false);
        assert_eq!(Foo::is_valid_discriminant(63), false);
    }
    {
        use self::many_ranges_b::Foo;
        assert_eq!(Foo::is_valid_discriminant(0), true);
        assert_eq!(Foo::is_valid_discriminant(1), false);
        assert_eq!(Foo::is_valid_discriminant(2), false);
        assert_eq!(Foo::is_valid_discriminant(39), false);
        assert_eq!(Foo::is_valid_discriminant(40), true);
        assert_eq!(Foo::is_valid_discriminant(41), true);
        assert_eq!(Foo::is_valid_discriminant(42), false);
        assert_eq!(Foo::is_valid_discriminant(43), false);
        assert_eq!(Foo::is_valid_discriminant(58), false);
        assert_eq!(Foo::is_valid_discriminant(59), false);
        assert_eq!(Foo::is_valid_discriminant(60), true);
        assert_eq!(Foo::is_valid_discriminant(62), false);
        assert_eq!(Foo::is_valid_discriminant(63), false);
    }
}

// This also tests what happens between dynamic libraries.
#[test]
fn transmuting_enums() {
    unsafe {
        use self::{command_a::Foo as FooA, command_c::Foo as FooC};

        let mut variant_a = NonExhaustive::new(FooC::A).transmute_enum::<FooA>();
        let mut variant_b = NonExhaustive::new(FooC::B(11)).transmute_enum::<FooA>();
        let mut variant_c = NonExhaustive::new(FooC::C).transmute_enum::<FooA>();
        let mut variant_d = FooC::D {
            name: "what".into(),
        }
        .piped(NonExhaustive::new)
        .transmute_enum::<FooA>();

        assert_eq!(variant_c.is_valid_discriminant(), false);
        assert_eq!(variant_d.is_valid_discriminant(), false);

        assert_eq!(variant_a.as_enum(), Ok(&FooA::A));
        assert_eq!(variant_b.as_enum(), Ok(&FooA::B(11)));
        assert_eq!(variant_c.as_enum().ok(), None);
        assert_eq!(variant_d.as_enum().ok(), None);

        assert_eq!(variant_a.as_enum_mut(), Ok(&mut FooA::A));
        assert_eq!(variant_b.as_enum_mut(), Ok(&mut FooA::B(11)));
        assert_eq!(variant_c.as_enum_mut().ok(), None);
        assert_eq!(variant_d.as_enum_mut().ok(), None);

        assert_eq!(variant_a.into_enum(), Ok(FooA::A));
        assert_eq!(variant_b.into_enum(), Ok(FooA::B(11)));
        assert_eq!(variant_c.into_enum().ok(), None);
        assert_eq!(variant_d.into_enum().ok(), None);
    }
}

#[test]
fn clone_test() {
    use self::generic_a::Foo;

    let arc = Arc::new(100);
    assert_eq!(Arc::strong_count(&arc), 1);

    let variant_a = NonExhaustive::new(Foo::<Arc<i32>>::A);
    let variant_b = NonExhaustive::new(Foo::<Arc<i32>>::B);
    let variant_c = NonExhaustive::new(Foo::<Arc<i32>>::C(arc.clone()));

    assert_eq!(Arc::strong_count(&arc), 2);

    assert_eq!(variant_a.clone(), variant_a);
    assert_eq!(variant_b.clone(), variant_b);
    {
        let clone_c = variant_c.clone();
        assert_eq!(Arc::strong_count(&arc), 3);
        assert_eq!(clone_c, variant_c);
    }
    assert_eq!(Arc::strong_count(&arc), 2);

    assert_eq!(variant_a, Foo::A);
    assert_eq!(variant_b, Foo::B);
    {
        let clone_c = variant_c.clone();
        assert_eq!(Arc::strong_count(&arc), 3);
        assert_eq!(clone_c, Foo::C(arc.clone()));
    }
    assert_eq!(Arc::strong_count(&arc), 2);

    drop(variant_c);
    assert_eq!(Arc::strong_count(&arc), 1);
}

#[test]
fn fmt_test() {
    use self::command_serde::Foo as FooC;

    let variant_a = FooC::A;
    let wrapped_a = NonExhaustive::new(variant_a.clone());

    let variant_b = FooC::B(11);
    let wrapped_b = NonExhaustive::new(variant_b.clone());

    let variant_c = FooC::C;
    let wrapped_c = NonExhaustive::new(variant_c.clone());

    let variant_d = FooC::D {
        name: "what".into(),
    };
    let wrapped_d = NonExhaustive::new(variant_d.clone());

    check_formatting_equivalence(&variant_a, &wrapped_a);
    check_formatting_equivalence(&variant_b, &wrapped_b);
    check_formatting_equivalence(&variant_c, &wrapped_c);
    check_formatting_equivalence(&variant_d, &wrapped_d);
}

#[test]
fn cmp_test() {
    use self::generic_a::Foo;

    let variant_a = Foo::<String>::A;
    let wrapped_a = NonExhaustive::new(variant_a.clone());

    let variant_b = Foo::<String>::B;
    let wrapped_b = NonExhaustive::new(variant_b.clone());

    let variant_c = Foo::<String>::C("what".into());
    let wrapped_c = NonExhaustive::new(variant_c.clone());

    for wrapped in [&wrapped_a, &wrapped_b, &wrapped_c] {
        assert_eq!(wrapped.cmp(wrapped), Ordering::Equal);
    }
    assert_eq!(wrapped_a.cmp(&wrapped_b), Ordering::Less);
    assert_eq!(wrapped_b.cmp(&wrapped_c), Ordering::Less);

    macro_rules! cmp_tests {
        (
            loop_variables=$variant:ident,$wrapped:ident,$which_one:ident;
            var_b=$var_b:ident;
            var_c=$var_c:ident;
        ) => {
            #[allow(unused_variables)]
            for ($variant, $wrapped) in [
                (&variant_a, &wrapped_a),
                (&variant_b, &wrapped_b),
                (&variant_c, &wrapped_c),
            ] {
                assert_eq!($wrapped == $which_one, true);
                assert_eq!($wrapped <= $which_one, true);
                assert_eq!($wrapped >= $which_one, true);
                assert_eq!($wrapped < $which_one, false);
                assert_eq!($wrapped > $which_one, false);
                assert_eq!($wrapped != $which_one, false);
                assert_eq!($wrapped.partial_cmp($which_one), Some(Ordering::Equal));
                assert_eq!($wrapped.eq($which_one), true);
                assert_eq!($wrapped.ne($which_one), false);
            }

            assert_eq!(wrapped_a == $var_b, false);
            assert_eq!(wrapped_a <= $var_b, true);
            assert_eq!(wrapped_a >= $var_b, false);
            assert_eq!(wrapped_a < $var_b, true);
            assert_eq!(wrapped_a > $var_b, false);
            assert_eq!(wrapped_a != $var_b, true);
            assert_eq!(wrapped_a.partial_cmp(&$var_b), Some(Ordering::Less));
            assert_eq!(wrapped_a.eq(&$var_b), false);
            assert_eq!(wrapped_a.ne(&$var_b), true);

            assert_eq!(wrapped_b == $var_c, false);
            assert_eq!(wrapped_b <= $var_c, true);
            assert_eq!(wrapped_b >= $var_c, false);
            assert_eq!(wrapped_b < $var_c, true);
            assert_eq!(wrapped_b > $var_c, false);
            assert_eq!(wrapped_b != $var_c, true);
            assert_eq!(wrapped_b.partial_cmp(&$var_c), Some(Ordering::Less));
            assert_eq!(wrapped_b.eq(&$var_c), false);
            assert_eq!(wrapped_b.ne(&$var_c), true);
        };
    }

    cmp_tests! {
        loop_variables=variant,wrapped,variant;
        var_b=variant_b;
        var_c=variant_c;
    }

    cmp_tests! {
        loop_variables=variant,wrapped,wrapped;
        var_b=wrapped_b;
        var_c=wrapped_c;
    }
}

#[test]
fn hash_test() {
    use self::generic_a::Foo;

    fn hash_value<H: Hash>(v: &H) -> u64 {
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }

    let variant_a = Foo::<String>::A;
    let wrapped_a = NonExhaustive::new(variant_a.clone());

    let variant_b = Foo::<String>::B;
    let wrapped_b = NonExhaustive::new(variant_b.clone());

    let variant_c = Foo::<String>::C("what".into());
    let wrapped_c = NonExhaustive::new(variant_c.clone());

    for (variant, wrapped) in [
        (&variant_a, &wrapped_a),
        (&variant_b, &wrapped_b),
        (&variant_c, &wrapped_c),
    ] {
        assert_eq!(hash_value(variant), hash_value(wrapped));
    }
}

#[test]
fn serde_test() {
    use self::command_serde::Foo as FooC;

    let variant_a = FooC::A;
    let variant_b = FooC::B(10);
    let variant_c = FooC::C;
    let variant_d = FooC::D {
        name: "what".into(),
    };

    let expected_a = NonExhaustive::new(variant_a.clone());
    let expected_b = NonExhaustive::new(variant_b.clone());
    let expected_c = NonExhaustive::new(variant_c.clone());
    let expected_d = NonExhaustive::new(variant_d.clone());

    let json_a = r#""A""#;
    let json_dd_a = serde_json::to_string(&json_a).unwrap();

    let json_b = r#"{"B":10}"#;
    let json_dd_b = serde_json::to_string(&json_b).unwrap();

    let json_c = r#""C""#;
    let json_dd_c = serde_json::to_string(&json_c).unwrap();

    let json_d = r#"{"D":{"name":"what"}}"#;
    let json_dd_d = serde_json::to_string(&json_d).unwrap();

    assert_eq!(
        serde_json::from_str::<NonExhaustiveFor<FooC>>(r#" "oinoiasnd" "#).map_err(drop),
        Err(()),
    );

    assert_eq!(
        NonExhaustiveFor::<FooC>::deserialize_from_proxy(r#"oinoiasnd"#.into()).map_err(drop),
        Err(()),
    );

    for (json_dd, json, expected, variant) in [
        (&*json_dd_a, json_a, &expected_a, &variant_a),
        (&*json_dd_b, json_b, &expected_b, &variant_b),
        (&*json_dd_c, json_c, &expected_c, &variant_c),
        (&*json_dd_d, json_d, &expected_d, &variant_d),
    ] {
        {
            let deserialized = serde_json::from_str::<NonExhaustiveFor<FooC>>(json_dd).unwrap();
            assert_eq!(deserialized, *expected);
            assert_eq!(deserialized, *variant);
        }
        {
            let deserialized =
                NonExhaustiveFor::<FooC>::deserialize_from_proxy(json.into()).unwrap();
            assert_eq!(deserialized, *expected);
            assert_eq!(deserialized, *variant);
        }

        assert_eq!(&*serde_json::to_string(&expected).unwrap(), json_dd);
        assert_eq!(&*expected.serialize_into_proxy().unwrap(), json);
        assert_eq!(&*serde_json::to_string(&variant).unwrap(), json);
    }
}
