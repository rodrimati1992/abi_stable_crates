#![allow(non_camel_case_types)]
#![cfg_attr(feature = "nightly_const_params", feature(const_generics))]
#![cfg_attr(miri, allow(unused_imports))]


mod layout_tests {

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod value;
    #[cfg(all(test,not(feature="only_new_tests")))]
    mod prefix_types;
    #[cfg(all(test,not(feature="only_new_tests")))]
    mod erased_types;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod pointer_types;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod repr_and_discr;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod sabi_trait;

    mod nonexhaustive_enums;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod get_static_equivalent;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod extra_checks_combined;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod stable_abi_attributes;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod const_params;

    #[cfg(all(test,not(feature="only_new_tests")))]
    mod lifetime_indices_tests;

    // #[cfg(test)]
    #[cfg(all(test,not(feature="only_new_tests")))]
    mod get_type_layout;


    #[cfg(all(test,not(feature="only_new_tests")))]
    mod shared_types;    
}
