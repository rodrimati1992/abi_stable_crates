/*!
Examples of `#[sabi_trait]` generated trait objects,for the documentation.
*/

use crate::sabi_trait;

/// An example trait,used to show what `#[sabi_trait]` generates in the docs.
#[sabi_trait]
#[sabi(use_dyn_trait)]
pub trait ConstExample:Debug+Clone{
    fn next_number(&self,num:usize)->usize;
}

impl ConstExample for usize{
    fn next_number(&self,num:usize)->usize{
        self+num
    }
}



#[sabi_trait]
#[doc(hidden)]
pub trait DocHiddenTrait{}
