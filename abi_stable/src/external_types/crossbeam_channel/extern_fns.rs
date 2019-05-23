use super::*;

use crate::traits::IntoReprC;

macro_rules! shared_fns {
    ( 
        erased=$erased:ident
        unerased=$unerased:ident 
    ) => (
        
        impl<T> $erased<T>{
            pub(super)fn from_unerased_value(value:$unerased<T>)->RBox<Self>{
                unsafe{
                    let boxed=RBox::new(value);
                    ErasedType::from_unerased(boxed)
                }
            }

            fn run<'a,F,R>(&'a self,f:F)->R
            where F:FnOnce(&'a $unerased<T>)->R
            {
                extern_fn_panic_handling!{
                    unsafe{
                        Self::run_as_unerased(self,f)
                    }
                }
            }
            
            pub(super) extern "C" fn clone(this:&Self) -> RBox<Self>{
                this.run(|this|{
                    Self::from_unerased_value(this.clone())
                })
            }
            pub(super) extern "C" fn is_empty(this:&Self) -> bool{
                this.run(|this|{
                    this.is_empty()
                })
            }
            pub(super) extern "C" fn is_full(this:&Self) -> bool{
                this.run(|this|{
                    this.is_full()
                })
            }
            pub(super) extern "C" fn len(this:&Self) -> usize{
                this.run(|this|{
                    this.len()
                })
            }
            pub(super) extern "C" fn capacity(this:&Self) -> ROption<usize>{
                this.run(|this|{
                    this.capacity().into_c()
                })
            }
        }

    )
}


shared_fns!{
    erased=ErasedSender
    unerased=Sender
}


shared_fns!{
    erased=ErasedReceiver
    unerased=Receiver
}


#[inline]
fn rresult_from<T,E0,E1>(res:Result<T,E0>)->RResult<T,E1>
where
    E0:Into<E1>
{
    match res {
        Ok(x)=>ROk(x),
        Err(e)=>RErr(e.into()),
    }
}


impl<T> ErasedSender<T>{
    pub(super) extern "C" fn send(this:&Self,val:T) -> RResult<(),RSendError<T>>{
        this.run(|this|{
            this.send(val).piped(rresult_from)
        })
    }
    pub(super) extern "C" fn try_send(this:&Self,val:T) -> RResult<(),RTrySendError<T>>{
        this.run(|this|{
            this.try_send(val).piped(rresult_from)
        })
    }
    pub(super) extern "C" fn send_timeout(
        this:&Self,
        val:T,
        timeout:RDuration,
    ) -> RResult<(),RSendTimeoutError<T>>{
        this.run(|this|{
            this.send_timeout(val,timeout.into()).piped(rresult_from)
        })
    }
}


impl<T> ErasedReceiver<T>{
    pub(super) extern "C" fn recv(this:&Self) -> RResult<T,RRecvError>{
        this.run(|this|{
            this.recv().piped(rresult_from)
        })
    }
    pub(super) extern "C" fn try_recv(this:&Self) -> RResult<T,RTryRecvError>{
        this.run(|this|{
            this.try_recv().piped(rresult_from)
        })
    }
    pub(super) extern "C" fn recv_timeout(
        this:&Self,
        timeout:RDuration,
    ) -> RResult<T,RRecvTimeoutError>{
        this.run(|this|{
            this.recv_timeout(timeout.into()).piped(rresult_from)
        })
    }
}

