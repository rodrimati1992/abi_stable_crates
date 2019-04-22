/*!
Types used to represent values at compile-time,eg:True/False.
*/

/**
Type-level booleans.

This is a re-export from `core_extensions::type_level_bool`,
so as to allow glob imports (`abi_stable::type_level_bool::*`)
without worrying about importing too many items.
*/
pub mod bools{
    #[doc(inline)]
    pub use core_extensions::type_level_bool::{True,False,Boolean};
}
