use super::*;

use crate::traits::IntoReprC;

macro_rules! shared_fns {
    (
        erased=$erased:ident
        unerased=$unerased:ident
    ) => {
        impl<T> $erased<T> {
            pub(super) fn from_unerased_value(value: $unerased<T>) -> RBox<Self> {
                unsafe {
                    let boxed = RBox::new(value);
                    ErasedType::from_unerased(boxed)
                }
            }

            fn run<'a, F, R>(this: RRef<'a, Self>, f: F) -> R
            where
                F: FnOnce(&'a $unerased<T>) -> R,
            {
                extern_fn_panic_handling! {
                    unsafe{
                        f(Self::downcast_into(this).get())
                    }
                }
            }

            pub(super) extern "C" fn clone(this: RRef<'_, Self>) -> RBox<Self> {
                Self::run(this, |this| Self::from_unerased_value(this.clone()))
            }
            pub(super) extern "C" fn is_empty(this: RRef<'_, Self>) -> bool {
                Self::run(this, |this| this.is_empty())
            }
            pub(super) extern "C" fn is_full(this: RRef<'_, Self>) -> bool {
                Self::run(this, |this| this.is_full())
            }
            pub(super) extern "C" fn len(this: RRef<'_, Self>) -> usize {
                Self::run(this, |this| this.len())
            }
            pub(super) extern "C" fn capacity(this: RRef<'_, Self>) -> ROption<usize> {
                Self::run(this, |this| this.capacity().into_c())
            }
        }
    };
}

shared_fns! {
    erased=ErasedSender
    unerased=Sender
}

shared_fns! {
    erased=ErasedReceiver
    unerased=Receiver
}

#[inline]
fn rresult_from<T, E0, E1>(res: Result<T, E0>) -> RResult<T, E1>
where
    E0: Into<E1>,
{
    match res {
        Ok(x) => ROk(x),
        Err(e) => RErr(e.into()),
    }
}

impl<T> ErasedSender<T> {
    pub(super) extern "C" fn send(this: RRef<'_, Self>, val: T) -> RResult<(), RSendError<T>> {
        Self::run(this, |this| this.send(val).piped(rresult_from))
    }
    pub(super) extern "C" fn try_send(
        this: RRef<'_, Self>,
        val: T,
    ) -> RResult<(), RTrySendError<T>> {
        Self::run(this, |this| this.try_send(val).piped(rresult_from))
    }
    pub(super) extern "C" fn send_timeout(
        this: RRef<'_, Self>,
        val: T,
        timeout: RDuration,
    ) -> RResult<(), RSendTimeoutError<T>> {
        Self::run(this, |this| {
            this.send_timeout(val, timeout.into()).piped(rresult_from)
        })
    }
}

impl<T> ErasedReceiver<T> {
    pub(super) extern "C" fn recv(this: RRef<'_, Self>) -> RResult<T, RRecvError> {
        Self::run(this, |this| this.recv().piped(rresult_from))
    }
    pub(super) extern "C" fn try_recv(this: RRef<'_, Self>) -> RResult<T, RTryRecvError> {
        Self::run(this, |this| this.try_recv().piped(rresult_from))
    }
    pub(super) extern "C" fn recv_timeout(
        this: RRef<'_, Self>,
        timeout: RDuration,
    ) -> RResult<T, RRecvTimeoutError> {
        Self::run(this, |this| {
            this.recv_timeout(timeout.into()).piped(rresult_from)
        })
    }
}
