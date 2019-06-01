use super::*;

/**
Converts a `&PhantomData<_Self>` to `&_Self`,and calls a function with that reference.

# Safety

The type behind the reference must actually be `_Self`.

*/
#[inline]
pub unsafe fn sabi_from_ref<'a,_Self,F,R>(this:&'a ErasedObject<_Self>,f:F)->R
where
    F:FnOnce(&'a _Self)->R,
{
    extern_fn_panic_handling!{no_early_return;
        let _self=__sabi_re::transmute_reference::<_,_Self>(this);
        f(_self)
    }
}

/**
Converts a `&mut PhantomData<_Self>` to `&mut _Self`,
and calls a function with that reference.

# Safety

The type behind the mutable reference must actually be `_Self`.

*/
#[inline]
pub unsafe fn sabi_from_mut<'a,_Self,F,R>(this:&'a mut ErasedObject<_Self>,f:F)->R
where
    F:FnOnce(&'a mut _Self)->R,
{
    extern_fn_panic_handling!{no_early_return;
        let _self=__sabi_re::transmute_mut_reference::<_,_Self>(this);
        f(_self)
    }
}


