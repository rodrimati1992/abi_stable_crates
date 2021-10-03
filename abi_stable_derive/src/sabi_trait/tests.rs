use crate::derive_sabi_trait_str as derive_sabi_trait;

use as_derive_utils::test_framework::Tests;

#[test]
fn must_not_pass() {
    let list = vec![
        "
            trait Foo {
                fn foo();
            }
        ",
        "
            trait Bar {
                fn bar(&self, x: &Self);
            }
        ",
        "
            trait Baz {
                fn bar(&self) -> Self;
            }
        ",
        // Can't return Self,at least for now.
        "
            trait Baz {
                fn bar(self) -> Self;
            }
        ",
        "
            trait Bar {
                const X: usize;
            }
        ",
    ];
    for elem in list {
        if derive_sabi_trait(elem).is_ok() {
            panic!("This passed wrongly:\n{}\n", elem);
        }
    }
}

#[test]
fn sabi_trait_test_cases() {
    Tests::load("sabi_trait").run_test(derive_sabi_trait);
}

#[test]
fn must_pass() {
    let list = vec![
        "
            trait ConstBaz<const N:usize> {
                fn baz(self);
            }
        ",
    ];

    for elem in list {
        derive_sabi_trait(elem).unwrap();
    }
}
