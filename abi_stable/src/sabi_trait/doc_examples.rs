/*!
Examples of `#[sabi_trait]` generated trait objects,for the documentation.
*/

use crate::sabi_trait;

#[sabi_trait]
#[sabi(use_dyn_trait)]
// #[sabi(debug_print_trait)]
pub trait ConstExample:Debug+Clone{
    fn next_number(&self,num:usize)->usize;
}

impl ConstExample for usize{
    fn next_number(&self,num:usize)->usize{
        self+num
    }
}

