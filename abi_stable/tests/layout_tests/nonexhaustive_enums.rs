use abi_stable::{
    abi_stability::abi_checking::{
        check_layout_compatibility_with_globals, AbiInstability, AbiInstabilityErrors,
        CheckingGlobals,
    },
    nonexhaustive_enum::{
        examples::{
            command_a, command_a_exhaustive, command_b, command_c, command_c_mismatched_field,
            command_h, command_h_mismatched_discriminant, command_one, command_one_more_traits_1,
            command_one_more_traits_2, command_one_more_traits_3, too_large,
        },
        NonExhaustiveFor,
    },
    type_layout::TypeLayout,
    StableAbi,
};

use core_extensions::{matches, SelfOps};

mod with_2_enums_a {
    use super::*;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Struct {
        a: command_a::Foo_NE,
        b: command_a::Foo_NE,
    }
}

mod with_2_enums_b {
    use super::*;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Struct {
        a: command_a::Foo_NE,
        b: command_b::Foo_NE,
    }
}

mod with_2_enums_c {
    use super::*;
    #[repr(C)]
    #[derive(abi_stable::StableAbi)]
    pub struct Struct {
        a: command_a::Foo_NE,
        b: command_c::Foo_NE,
    }
}

#[cfg(not(miri))]
fn check_subsets<F>(list: &[&'static TypeLayout], mut f: F)
where
    F: FnMut(&[AbiInstability]),
{
    let globals = CheckingGlobals::new();
    for (l_i, l_abi) in list.iter().enumerate() {
        for (r_i, r_abi) in list.iter().enumerate() {
            let res = check_layout_compatibility_with_globals(l_abi, r_abi, &globals);

            if l_i <= r_i {
                assert_eq!(res, Ok(()), "\n\nl_i:{} r_i:{}\n\n", l_i, r_i);
            } else {
                if let Ok(_) = res {
                    let _ = dbg!(l_i);
                    let _ = dbg!(r_i);
                }
                let errs = res.unwrap_err().flatten_errors();

                f(&*errs);
            }
        }
    }
}

#[cfg(not(miri))]
#[test]
fn check_enum_subsets() {
    let list = vec![
        <NonExhaustiveFor<command_a::Foo> as StableAbi>::LAYOUT,
        <NonExhaustiveFor<command_b::Foo> as StableAbi>::LAYOUT,
        <NonExhaustiveFor<command_c::Foo> as StableAbi>::LAYOUT,
    ];

    check_subsets(&list, |errs| {
        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::TooManyVariants { .. })));
    })
}

#[cfg(miri)]
#[test]
fn check_enum_subsets() {
    let globals = CheckingGlobals::new();

    let inter0 = <NonExhaustiveFor<command_a::Foo> as StableAbi>::LAYOUT;
    let inter1 = <NonExhaustiveFor<command_b::Foo> as StableAbi>::LAYOUT;

    assert_eq!(
        check_layout_compatibility_with_globals(inter0, inter1, &globals),
        Ok(())
    );
    assert_ne!(
        check_layout_compatibility_with_globals(inter1, inter0, &globals),
        Ok(())
    );
}

// This test ensures that a struct with 2 nonexhaustive enums works as expected.
//
// This test is partly to ensure that a `NonExhaustive<>` produces different
// `UTypeId`s with different enums,
// that is a bug I discovered while testing out type errors in
// the 2_nonexhaustive example crates.
// This bug was caused by `#[sabi(unconstrained(T))]`
// causing the type parameter to be ignored when generating the UTypeId,
// meaning that even if the type parameter changed the UTypeId wouldn't.
#[cfg(not(miri))]
#[test]
fn check_2_enum_subsets() {
    let list = vec![
        <with_2_enums_a::Struct as StableAbi>::LAYOUT,
        <with_2_enums_b::Struct as StableAbi>::LAYOUT,
        <with_2_enums_c::Struct as StableAbi>::LAYOUT,
    ];

    check_subsets(&list, |errs| {
        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::TooManyVariants { .. })));
    })
}

#[cfg(miri)]
#[test]
fn check_2_enum_subsets() {
    let globals = CheckingGlobals::new();

    let inter0 = <with_2_enums_a::Struct as StableAbi>::LAYOUT;
    let inter1 = <with_2_enums_b::Struct as StableAbi>::LAYOUT;

    assert_eq!(
        check_layout_compatibility_with_globals(inter0, inter1, &globals),
        Ok(())
    );
    assert_ne!(
        check_layout_compatibility_with_globals(inter1, inter0, &globals),
        Ok(())
    );
}

#[cfg(not(miri))]
#[test]
fn check_impld_traits_subsets() {
    let list = vec![
        <NonExhaustiveFor<command_one::Foo> as StableAbi>::LAYOUT,
        <NonExhaustiveFor<command_one_more_traits_1::Foo> as StableAbi>::LAYOUT,
        <NonExhaustiveFor<command_one_more_traits_2::Foo> as StableAbi>::LAYOUT,
        <NonExhaustiveFor<command_one_more_traits_3::Foo> as StableAbi>::LAYOUT,
    ];

    check_subsets(&list, |errs| {
        assert!(errs
            .iter()
            .any(|err| matches!(err, AbiInstability::ExtraCheckError { .. })));
    })
}

#[cfg(miri)]
#[test]
fn check_impld_traits_subsets() {
    let globals = CheckingGlobals::new();

    let inter0 = <NonExhaustiveFor<command_one::Foo> as StableAbi>::LAYOUT;
    let inter1 = <NonExhaustiveFor<command_one_more_traits_1::Foo> as StableAbi>::LAYOUT;

    assert_eq!(
        check_layout_compatibility_with_globals(inter0, inter1, &globals),
        Ok(())
    );
    assert_ne!(
        check_layout_compatibility_with_globals(inter1, inter0, &globals),
        Ok(())
    );
}

#[test]
fn exhaustiveness() {
    let globals = CheckingGlobals::new();

    let unwrapped = <command_a_exhaustive::Foo as StableAbi>::LAYOUT;
    let wrapped = <NonExhaustiveFor<command_a::Foo> as StableAbi>::LAYOUT;

    for (l, r) in vec![(unwrapped, wrapped), (wrapped, unwrapped)] {
        check_layout_compatibility_with_globals(l, r, &globals)
            .unwrap_err()
            .flatten_errors()
            .iter()
            .any(|err| matches!(err, AbiInstability::MismatchedExhaustiveness { .. }));
    }
}

#[test]
fn mismatched_discriminant() {
    let globals = CheckingGlobals::new();

    let regular = <NonExhaustiveFor<command_h::Foo> as StableAbi>::LAYOUT;
    let mismatched =
        <NonExhaustiveFor<command_h_mismatched_discriminant::Foo> as StableAbi>::LAYOUT;

    check_layout_compatibility_with_globals(regular, mismatched, &globals)
        .unwrap_err()
        .flatten_errors()
        .iter()
        .any(|err| matches!(err, AbiInstability::EnumDiscriminant { .. }));
}

#[test]
fn check_storage_unstorable() {
    let globals = CheckingGlobals::new();

    let abi_a = <NonExhaustiveFor<command_a::Foo> as StableAbi>::LAYOUT;
    #[cfg(not(miri))]
    let abi_b = <NonExhaustiveFor<command_b::Foo> as StableAbi>::LAYOUT;
    let abi_large = <NonExhaustiveFor<too_large::Foo> as StableAbi>::LAYOUT;

    #[cfg(not(miri))]
    let checks = vec![
        (abi_a, abi_large),
        (abi_b, abi_large),
        (abi_large, abi_large),
        (abi_large, abi_a),
        (abi_large, abi_b),
    ];

    #[cfg(miri)]
    let checks = vec![(abi_a, abi_large)];

    for (l, r) in checks {
        check_layout_compatibility_with_globals(l, r, &globals)
            .unwrap_err()
            .flatten_errors()
            .iter()
            .any(|err| matches!(err, AbiInstability::IncompatibleWithNonExhaustive { .. }));
    }
}

#[test]
fn incompatible_overlapping_variants() {
    let abi_one = <NonExhaustiveFor<command_one::Foo> as StableAbi>::LAYOUT;
    let abi_a = <NonExhaustiveFor<command_a::Foo> as StableAbi>::LAYOUT;
    let abi_b = <NonExhaustiveFor<command_b::Foo> as StableAbi>::LAYOUT;
    let abi_c = <NonExhaustiveFor<command_c::Foo> as StableAbi>::LAYOUT;
    let abi_c_mf = <NonExhaustiveFor<command_c_mismatched_field::Foo> as StableAbi>::LAYOUT;

    fn unwrap_the_err(errs: Result<(), AbiInstabilityErrors>) {
        let mut found_mismatch = false;
        for e in errs.clone().unwrap_err().flatten_errors().into_iter() {
            if let AbiInstability::Name(ef) = &e {
                found_mismatch = true;
                for full_type in vec![&ef.expected, &ef.found] {
                    assert!(
                        full_type.name() == "RVec" || full_type.name() == "RString",
                        "err:{:?}",
                        e
                    );
                }
            }
        }
        assert!(found_mismatch, "errs:{:#?}", errs);
    }

    #[cfg(not(miri))]
    {
        let globals = CheckingGlobals::new();
        check_layout_compatibility_with_globals(abi_a, abi_b, &globals).unwrap();
        check_layout_compatibility_with_globals(abi_b, abi_c, &globals).unwrap();
        check_layout_compatibility_with_globals(abi_b, abi_c_mf, &globals).piped(unwrap_the_err);
    }
    {
        let globals = CheckingGlobals::new();
        check_layout_compatibility_with_globals(abi_a, abi_b, &globals).unwrap();

        check_layout_compatibility_with_globals(abi_b, abi_c_mf, &globals).unwrap();

        check_layout_compatibility_with_globals(abi_b, abi_c, &globals).piped(unwrap_the_err);
    }
    #[cfg(not(miri))]
    {
        let globals = CheckingGlobals::new();
        check_layout_compatibility_with_globals(abi_one, abi_c, &globals).unwrap();
        assert_eq!(globals.nonexhaustive_map.lock().unwrap().value_len(), 1);
        check_layout_compatibility_with_globals(abi_a, abi_b, &globals).unwrap();
        assert_eq!(globals.nonexhaustive_map.lock().unwrap().value_len(), 2);
        check_layout_compatibility_with_globals(abi_one, abi_a, &globals).unwrap();
        assert_eq!(globals.nonexhaustive_map.lock().unwrap().value_len(), 1);
    }
}
