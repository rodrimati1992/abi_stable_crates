//! Examples of [`#[sabi_trait]`](macro@crate::sabi_trait)
//! generated trait objects,for the documentation.

use crate::sabi_trait;

#[sabi_trait]
/// An example trait, used to show what [`#[sabi_trait]`](macro@crate::sabi_trait)
/// generates in the docs.
#[sabi(use_dyn_trait)]
pub trait ConstExample: Debug + Clone {
    #[sabi(last_prefix_field)]
    fn next_number(&self, num: usize) -> usize;
}

impl ConstExample for usize {
    fn next_number(&self, num: usize) -> usize {
        self + num
    }
}

#[sabi_trait]
// #[sabi(debug_print_trait)]
/// An example trait object that uses `RObject` as a backend.
pub trait Doer: Debug {
    fn value(&self) -> usize;

    fn do_it(&self, num: usize) -> usize;

    #[sabi(last_prefix_field)]
    fn add_into(&mut self, num: usize);
}

impl Doer for usize {
    fn value(&self) -> usize {
        *self
    }

    fn do_it(&self, num: usize) -> usize {
        self + num
    }
    fn add_into(&mut self, num: usize) {
        *self += num;
    }
}

#[sabi_trait]
#[doc(hidden)]
pub trait DocHiddenTrait {}

//////////////////////////////////////////

/// The trait used in examples of [`#[sabi_trait]`](macro@crate::sabi_trait)
/// trait object methods,
/// in [`abi_stable::docs::sabi_trait_inherent`]
#[abi_stable::sabi_trait]
// #[sabi(debug_print_trait)]
pub trait Action: Debug {
    /// Gets the current value of `self`.
    fn get(&self) -> usize;

    /// Adds `val` into `self`, returning the new value.
    fn add_mut(&mut self, val: usize) -> usize;

    /// Adds `val` into `self`, returning the new value.
    #[sabi(last_prefix_field)]
    fn add_into(self, val: usize) -> usize;
}

impl Action for usize {
    fn get(&self) -> usize {
        *self
    }
    fn add_mut(&mut self, val: usize) -> usize {
        *self += val;
        *self
    }
    fn add_into(mut self, val: usize) -> usize {
        self += val;
        self
    }
}
