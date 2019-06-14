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

#[cfg(test)]
mod sabi_trait;
