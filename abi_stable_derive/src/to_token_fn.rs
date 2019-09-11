use proc_macro2::TokenStream;
use quote::ToTokens;

use std::cell::RefCell;

/// Used to more easily implement ToTokens.
pub(crate) struct ToTokenFnMut<F> {
    func: RefCell<F>,
}

impl<F> ToTokenFnMut<F>
where
    F: FnMut(&mut TokenStream),
{
    pub(crate) fn new(f: F) -> Self {
        Self {
            func: RefCell::new(f),
        }
    }
    #[allow(dead_code)]
    pub(crate) fn boxed<'a>(f: F) -> Box<dyn ToTokens+'a> 
    where
        F:'a,
    {
        Box::new(Self::new(f))
    }
}

impl<F> ToTokens for ToTokenFnMut<F>
where
    F: FnMut(&mut TokenStream),
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut func = RefCell::borrow_mut(&self.func);
        (&mut *func)(tokens);
    }
}
