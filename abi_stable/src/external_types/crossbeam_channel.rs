/*!

Ffi-safe wrapper types around the 
[crossbeam-channel](https://crates.io/crates/crossbeam-channel/) 
channel types.

*/

use std::{
    fmt::{self,Debug},
    marker::PhantomData,
    time::Duration,
};

use crossbeam_channel::{
    Receiver,
    Sender,
    SendError,
    RecvError,
    TrySendError,
    TryRecvError,
    SendTimeoutError,
    RecvTimeoutError,
};

use core_extensions::SelfOps;

use crate::{
    marker_type::UnsafeIgnoredType,
    traits::{IntoReprC,IntoReprRust,ErasedType},
    std_types::{RResult,ROk,RErr,ROption,RDuration,RBox},
    prefix_type::{PrefixTypeTrait,WithMetadata},
};


mod errors;
mod extern_fns;
mod iteration;

#[cfg(all(test,not(feature="only_new_tests")))]
mod tests;

use self::{
    errors::{
        RSendError,
        RRecvError,
        RTrySendError,
        RTryRecvError,
        RSendTimeoutError,
        RRecvTimeoutError,
    }
};

pub use self::{
    iteration::{RIter,RIntoIter},
};

///////////////////////////////////////////////////////////////////////////////

/// Creates a receiver that can never receive any value.
pub fn never<T>() -> RReceiver<T>{
    crossbeam_channel::never::<T>().into()
}

/**
Creates a channel which can hold up to `capacity` elements in its internal queue.

If `capacity==0`,the value must be sent to a receiver in the middle of a `recv` call.

# Panics

Panics if `capacity >= usize::max_value() / 4`.

*/
pub fn bounded<T>(capacity:usize) -> (RSender<T>, RReceiver<T>) {
    let (tx,rx)=crossbeam_channel::bounded::<T>(capacity);
    ( tx.into(), rx.into() )
}

/**
Creates a channel which can hold an unbounded ammount elements in its internal queue.
*/
pub fn unbounded<T>() -> (RSender<T>, RReceiver<T>) {
    let (tx,rx)=crossbeam_channel::unbounded::<T>();
    ( tx.into(), rx.into() )
}


///////////////////////////////////////////////////////////////////////////////


/// The sender end of a channel,
/// which can be either bounded or unbounded.
pub struct RSender<T>{
    channel:RBox<ErasedSender<T>>,
    vtable:*const VTable<T>,
}


impl<T> RSender<T>{
    fn vtable<'a>(&self)->&'a VTable<T>{
        unsafe{ &*self.vtable }
    }

/**
Blocks until `value` is either sent,or the the other end is disconnected.

If the channel queue is full,this will block to send `value`.

If the channel is disconnected,this will return an error with `value`.
*/
    pub fn send(&self,value:T) -> Result<(),SendError<T>>{
        let vtable=self.vtable();

        vtable.send()(&*self.channel,value)
            .piped(result_from)
    }


/**
Immediately sends `value`,or returns with an error.

An error will be returned in these 2 conditions:

- the channel is full.

- the channel has been disconnected.

If the channel has a capacity of 0,it will only send `value` if 
the other end is calling `recv`.

*/
    pub fn try_send(&self,value:T) -> Result<(),TrySendError<T>>{
        let vtable=self.vtable();

        vtable.try_send()(&*self.channel,value)
            .piped(result_from)
    }

/**
Blocks until a timeout to send `value`.

An error will be returned in these 2 conditions:

- the value could not be sent before the timeout.

- the channel has been disconnected.

If the channel has a capacity of 0,it will only send `value` if 
the other end calls `recv` before the timeout.

*/
    pub fn send_timeout(&self,value:T,timeout:Duration) -> Result<(),SendTimeoutError<T>>{
        let vtable=self.vtable();

        vtable.send_timeout()(&*self.channel,value,timeout.into())
            .piped(result_from)
    }

    /// Returns true if there are no values in the channel queue.
    pub fn is_empty(&self) -> bool{
        let vtable=self.vtable();

        vtable.sender_is_empty()(&*self.channel)
    }

    /// Returns true if the channel queue is full.
    ///
    /// This always returns true for channels constructed with `bounded(0)`.
    pub fn is_full(&self) -> bool{
        let vtable=self.vtable();

        vtable.sender_is_full()(&*self.channel)
    }

    /// Returns the ammount of values in the channel queue.
    pub fn len(&self) -> usize{
        let vtable=self.vtable();
        
        vtable.sender_len()(&*self.channel)
    }

    /// Returns the ammount of values the channel queue can hold.
    /// 
    /// This returns None if the channel is unbounded.
    pub fn capacity(&self) -> Option<usize>{
        let vtable=self.vtable();
        
        vtable.sender_capacity()(&*self.channel)
            .into_rust()
    }

}


impl<T> Clone for RSender<T>{
    /// Clones this channel end,getting another handle into the channel.
    ///
    /// Note that this allocates an RBox<_>.
    fn clone(&self)->Self{
        let vtable=self.vtable();

        Self{
            channel:vtable.clone_sender()(&*self.channel),
            vtable:self.vtable,
        }
    }
}

impl<T> Debug for RSender<T>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt("RSender{..}",f)
    }
}

unsafe impl<T: Send> Sync for RSender<T>{}

unsafe impl<T: Send> Send for RSender<T>{}



impl_from_rust_repr! {
    impl[T] From<Sender<T>> for RSender<T> {
        fn(this){
            Self{
                channel:ErasedSender::from_unerased_value(this),
                vtable:MakeVTable::<T>::VTABLE.as_prefix()
            }
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


/// The receiver end of a channel,
/// which can be either bounded or unbounded.
pub struct RReceiver<T>{
    channel:RBox<ErasedReceiver<T>>,
    vtable:*const VTable<T>,
}


impl<T> RReceiver<T>{
    fn vtable<'a>(&self)->&'a VTable<T>{
        unsafe{ &*self.vtable }
    }

/**
Blocks until a value is either received,or the the other end is disconnected.

If the channel queue is empty,this will block to receive a value.

This will return an error if the channel is disconnected.
*/
    pub fn recv(&self) -> Result<T,RecvError>{
        let vtable=self.vtable();

        vtable.recv()(&*self.channel)
            .piped(result_from)
    }

/**
Immediately receives a value,or returns with an error.

An error will be returned in these 2 conditions:

- the channel is empty.

- the channel has been disconnected.

If the channel has a capacity of 0,it will only receive a value if 
the other end is calling `send`.

*/
    pub fn try_recv(&self) -> Result<T,TryRecvError>{
        let vtable=self.vtable();

        vtable.try_recv()(&*self.channel)
            .piped(result_from)
    }

/**
Blocks until a timeout to receive a value.

An error will be returned in these 2 conditions:

- A value could not be received before the timeout.

- the channel has been disconnected.

If the channel has a capacity of 0,it will only receive a value if 
the other end calls `send` before the timeout.

*/
    pub fn recv_timeout(&self,timeout:Duration) -> Result<T,RecvTimeoutError>{
        let vtable=self.vtable();

        vtable.recv_timeout()(&*self.channel,timeout.into())
            .piped(result_from)
    }

    /// Returns true if there are no values in the channel queue.
    pub fn is_empty(&self) -> bool{
        let vtable=self.vtable();

        vtable.receiver_is_empty()(&*self.channel)
    }

    /// Returns true if the channel queue is full.
    ///
    /// This always returns true for channels constructed with `bounded(0)`.
    pub fn is_full(&self) -> bool{
        let vtable=self.vtable();

        vtable.receiver_is_full()(&*self.channel)
    }

    /// Returns the ammount of values in the channel queue.
    pub fn len(&self) -> usize{
        let vtable=self.vtable();
        
        vtable.receiver_len()(&*self.channel)
    }

    /// Returns the ammount of values the channel queue can hold.
    /// 
    /// This returns None if the channel is unbounded.
    pub fn capacity(&self) -> Option<usize>{
        let vtable=self.vtable();
        
        vtable.receiver_capacity()(&*self.channel)
            .into_rust()
    }

    /// Creates an Iterator that receives values from the channel.
    pub fn iter(&self)->RIter<'_,T>{
        RIter{
            channel:self,
        }
    }
}

impl<'a, T> IntoIterator for &'a RReceiver<T> {
    type Item = T;
    type IntoIter = RIter<'a, T>;

    /// Creates an Iterator that receives values from the channel.
    #[inline]
    fn into_iter(self) -> RIter<'a, T> {
        self.iter()
    }
}

impl<T> IntoIterator for RReceiver<T> {
    type Item = T;
    type IntoIter = RIntoIter<T>;

    /// Creates an Iterator that receives values from the channel.
    #[inline]
    fn into_iter(self) -> RIntoIter<T> {
        RIntoIter { 
            channel: self 
        }
    }
}

impl<T> Clone for RReceiver<T>{
    /// Clones this channel end,getting another handle into the channel.
    ///
    /// Note that this allocates an RBox<_>.
    fn clone(&self)->Self{
        let vtable=self.vtable();

        Self{
            channel:vtable.clone_receiver()(&*self.channel),
            vtable:self.vtable,
        }
    }
}

impl<T> Debug for RReceiver<T>{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt("RReceiver{..}",f)
    }
}

unsafe impl<T: Send> Sync for RReceiver<T>{}

unsafe impl<T: Send> Send for RReceiver<T>{}


impl_from_rust_repr! {
    impl[T] From<Receiver<T>> for RReceiver<T> {
        fn(this){
            Self{
                channel:ErasedReceiver::from_unerased_value(this),
                vtable:MakeVTable::<T>::VTABLE.as_prefix()
            }
        }
    }
}


///////////////////////////////////////////////////////////////////////////////


#[inline]
fn result_from<T,E0,E1>(res:RResult<T,E0>)->Result<T,E1>
where
    E0:Into<E1>
{
    match res {
        ROk(x)=>Ok(x),
        RErr(e)=>Err(e.into()),
    }
}



#[repr(C)]
#[derive(StableAbi)]
struct ErasedSender<T>(
    PhantomData<T>,
    UnsafeIgnoredType<Sender<T>>,
);

unsafe impl<'a,T:'a> ErasedType<'a> for ErasedSender<T> {
    type Unerased=Sender<T>;
}

#[repr(C)]
#[derive(StableAbi)]
struct ErasedReceiver<T>(
    PhantomData<T>,
    UnsafeIgnoredType<Receiver<T>>,
);

unsafe impl<'a,T:'a> ErasedType<'a> for ErasedReceiver<T> {
    type Unerased=Receiver<T>;
}

///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="VTable")))]
#[sabi(missing_field(panic))]
struct VTableVal<T>{
    send:extern "C" fn(this:&ErasedSender<T>,T) -> RResult<(),RSendError<T>>,
    try_send:extern "C" fn(this:&ErasedSender<T>,T) -> RResult<(),RTrySendError<T>>,
    send_timeout:
        extern "C" fn(
            this:&ErasedSender<T>,
            value:T,
            timeout:RDuration
        ) -> RResult<(),RSendTimeoutError<T>>,
    clone_sender:extern "C" fn(this:&ErasedSender<T>) -> RBox<ErasedSender<T>>,
    sender_is_empty:extern "C" fn(this:&ErasedSender<T>) -> bool,
    sender_is_full:extern "C" fn(this:&ErasedSender<T>) -> bool,
    sender_len:extern "C" fn(this:&ErasedSender<T>) -> usize,
    sender_capacity:extern "C" fn(this:&ErasedSender<T>) -> ROption<usize>,

    
    recv:
        extern "C" fn(this:&ErasedReceiver<T>) -> RResult<T,RRecvError>,
    try_recv:
        extern "C" fn(this:&ErasedReceiver<T>) -> RResult<T,RTryRecvError>,
    #[sabi(last_prefix_field)]
    recv_timeout:
        extern "C" fn(
            this:&ErasedReceiver<T>, 
            timeout: RDuration
        ) -> RResult<T,RRecvTimeoutError>,
    clone_receiver:extern "C" fn(this:&ErasedReceiver<T>) -> RBox<ErasedReceiver<T>>,
    receiver_is_empty:extern "C" fn(this:&ErasedReceiver<T>) -> bool,
    receiver_is_full:extern "C" fn(this:&ErasedReceiver<T>) -> bool,
    receiver_len:extern "C" fn(this:&ErasedReceiver<T>) -> usize,
    receiver_capacity:extern "C" fn(this:&ErasedReceiver<T>) -> ROption<usize>,

}


struct MakeVTable<'a,T>(&'a T);


impl<'a,T:'a> MakeVTable<'a,T>{
    const VALUE:VTableVal<T>=VTableVal{
        send:ErasedSender::send,
        try_send:ErasedSender::try_send,
        send_timeout:ErasedSender::send_timeout,
        clone_sender:ErasedSender::clone,
        sender_is_empty:ErasedSender::is_empty,
        sender_is_full:ErasedSender::is_full,
        sender_len:ErasedSender::len,
        sender_capacity:ErasedSender::capacity,

        recv:ErasedReceiver::recv,
        try_recv:ErasedReceiver::try_recv,
        recv_timeout:ErasedReceiver::recv_timeout,
        clone_receiver:ErasedReceiver::clone,
        receiver_is_empty:ErasedReceiver::is_empty,
        receiver_is_full:ErasedReceiver::is_full,
        receiver_len:ErasedReceiver::len,
        receiver_capacity:ErasedReceiver::capacity,
    };

    // The VTABLE for this type in this executable/library
    const VTABLE: &'a WithMetadata<VTableVal<T>> = 
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VALUE);
}

