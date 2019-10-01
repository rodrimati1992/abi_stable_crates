#![allow(non_camel_case_types)]

#[cfg(all(test,not(feature="only_new_tests")))]
mod value;
#[cfg(all(test,not(feature="only_new_tests")))]
mod prefix_types;
#[cfg(all(test,not(feature="only_new_tests")))]
mod erased_types;

#[cfg(all(test,not(feature="only_new_tests")))]
// #[cfg(test)]
mod repr_and_discr;

#[cfg(all(test,not(feature="only_new_tests")))]
mod sabi_trait;

#[cfg(all(test,not(feature="only_new_tests")))]
mod nonexhaustive_enums;

#[cfg(all(test,not(feature="only_new_tests")))]
//#[cfg(test)]
mod get_static_equivalent;

#[cfg(all(test,not(feature="only_new_tests")))]
mod extra_checks_combined;

#[cfg(all(test,not(feature="only_new_tests")))]
mod stable_abi_attributes;

#[cfg(all(test,not(feature="only_new_tests")))]
mod const_params;

#[cfg(test)]
mod lifetime_indices_tests;

#[cfg(test)]
mod get_type_layout;


#[cfg(test)]
mod shared_types;
