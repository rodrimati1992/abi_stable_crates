use crate::derive_sabi_trait_str as derive_sabi_trait;

use abi_stable_shared::{file_span,test_utils::{must_panic}};


#[test]
fn must_not_pass(){

    let list=vec![
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
        "
    ];
    for elem in list {
        must_panic(file_span!(),||{
            derive_sabi_trait(elem).unwrap()
        }).unwrap_or_else(|_|{
            panic!("This passed wrongly:\n{}\n", elem);
        });
        
    }
}


#[test]
fn must_pass(){
    let list=vec![
        // uncomment once syn supports const parameters
        // "
        //     trait ConstBaz<const N:usize> {
        //         fn baz(self: Self);
        //     }
        // ",
    ];

    for elem in list {
        let _=derive_sabi_trait(elem);
    }
}