//! Contains an ffi-safe equivalent of `parking_lot::Mutex`.

use std::{
    cell::UnsafeCell,
    fmt::{self,Debug,Display},
    ops::{Deref,DerefMut},
    marker::PhantomData,
    mem,
};

use parking_lot::{RawMutex};
use lock_api::{
    RawMutex as RawMutexTrait,
    RawMutexTimed,
};

use super::{RAW_LOCK_SIZE,UnsafeOveralignedField};

use crate::{
    StableAbi,
    marker_type::UnsyncUnsend,
    prefix_type::{PrefixTypeTrait,WithMetadata},
    sabi_types::StaticRef,
    std_types::*,
};


///////////////////////////////////////////////////////////////////////////////

const OM_PADDING:usize=RAW_LOCK_SIZE-mem::size_of::<RawMutex>();

/**
The equivalent of a [`parking_lot::RawMutex`], but with a stable ABI.

[`parking_lot::RawMutex`]: https://docs.rs/parking_lot/0.9/parking_lot/struct.RawMutex.html
*/
#[derive(StableAbi)]
#[repr(C)]
pub struct RRawMutex {
    inner: UnsafeOveralignedField<RawMutex,[u8;OM_PADDING]>,
}

impl RRawMutex {
    const fn new() -> Self {
        Self {
            inner: UnsafeOveralignedField::new(<RawMutex as RawMutexTrait>::INIT,[0u8;OM_PADDING]),
        }
    }
}

unsafe impl RawMutexTrait for RRawMutex {
    type GuardMarker = <RawMutex as RawMutexTrait>::GuardMarker;
    const INIT: Self = Self::new();

    #[inline]
    fn lock(&self) {
        self.inner.value.lock()
    }

    #[inline]
    fn try_lock(&self) -> bool {
        self.inner.value.try_lock()
    }

    #[inline]
    fn unlock(&self) {
        self.inner.value.unlock()
    }
}

#[allow(dead_code)]
fn assert_mutex_size(){
    let _assert_size:[();RAW_LOCK_SIZE-mem::size_of::<RRawMutex>()];
    let _assert_size:[();mem::size_of::<RRawMutex>()-RAW_LOCK_SIZE];
}

/**
A mutual exclusion lock that allows dynamic mutable borrows of shared data.

# Poisoning 

As opposed to the standard library version of this type,
this mutex type does not use poisoning,
simply unlocking the lock when a panic happens.

# Example

```
use abi_stable::external_types::RMutex;

static MUTEX:RMutex<usize>=RMutex::new(0);

let guard=std::thread::spawn(||{
    for _ in 0..100 {
        *MUTEX.lock()+=1;
    }
});

for _ in 0..100 {
    *MUTEX.lock()+=1;
}

guard.join().unwrap();

assert_eq!(*MUTEX.lock(),200);

```
*/
#[repr(C)]
#[derive(StableAbi)]
pub struct RMutex<T>{
    raw_mutex:RRawMutex,
    data:UnsafeCell<T>,
    vtable:StaticRef<VTable>,
}


/**
A mutex guard,which allows mutable access to the data inside the mutex.

When dropped this will unlock the mutex.

*/
#[repr(transparent)]
#[derive(StableAbi)]
#[sabi(bound="T:'a")]
#[must_use]
pub struct RMutexGuard<'a, T> {
    rmutex: &'a RMutex<T>,
    _marker: PhantomData<Tuple2<&'a mut T, UnsyncUnsend>>,
}



///////////////////////////////////////////////////////////////////////////////


impl<T> RMutex<T>{
    /// Constructs a mutex,wrapping `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// static MUTEX:RMutex<Option<String>>=RMutex::new(None);
    /// 
    /// let mutex=RMutex::new(0);
    ///
    /// ```
    pub const fn new(value:T)->Self{
        Self{
            raw_mutex:RRawMutex::INIT,
            data:UnsafeCell::new(value),
            vtable:WithMetadata::as_prefix(VTable::VTABLE),
        }
    }

    #[inline]
    fn vtable(&self)->&'static VTable{
        self.vtable.get()
    }

    #[inline]
    fn make_guard(&self)->RMutexGuard<'_,T>{
        RMutexGuard{
            rmutex:self,
            _marker:PhantomData
        }
    }

    /// Unwraps this mutex into its wrapped data.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// let mutex=RMutex::new("hello".to_string());
    ///
    /// assert_eq!(mutex.into_inner().as_str(),"hello");
    ///
    /// ```
    #[inline]
    pub fn into_inner(self)->T{
        self.data.into_inner()
    }

    /// Gets a mutable reference to its wrapped data.
    ///
    /// This does not require any locking,since it takes `self` mutably.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::external_types::RMutex;
    ///
    /// let mut mutex=RMutex::new("Hello".to_string());
    ///
    /// mutex.get_mut().push_str(", World!");
    ///
    /// assert_eq!(mutex.lock().as_str(),"Hello, World!");
    /// 
    ///
    /// ```
    #[inline]
    pub fn get_mut(&mut self)->RMutexGuard<'_,T>{
        self.make_guard()
    }

    /**
Acquires a mutex,blocking the current thread until it can.

This function returns a guard which releases the mutex when it is dropped.

Trying to lock the mutex in the same thread that holds the lock will cause a deadlock.

# Example

```
use abi_stable::external_types::RMutex;

static MUTEX:RMutex<usize>=RMutex::new(0);

let guard=std::thread::spawn(|| *MUTEX.lock()+=1 );

*MUTEX.lock()+=4;

guard.join().unwrap();

assert_eq!(*MUTEX.lock(),5);

```

    */
    #[inline]
    pub fn lock(&self)->RMutexGuard<'_,T>{
        self.vtable().lock()(&self.raw_mutex);
        self.make_guard()
    }
    /**
Attemps to acquire a mutex.

Returns the mutex guard if the mutex can be immediately acquired,otherwise returns RNone.

# Example

```
use abi_stable::external_types::RMutex;

static MUTEX:RMutex<usize>=RMutex::new(0);

let mut guard=MUTEX.try_lock().unwrap();

assert!( MUTEX.try_lock().is_none() );

assert_eq!(*guard,0);

```

*/    
    #[inline]
    pub fn try_lock(&self) -> ROption<RMutexGuard<'_,T>>{
        if self.vtable().try_lock()(&self.raw_mutex) {
            RSome(self.make_guard())
        }else{
            RNone
        }
    }
    
/**
Attempts to acquire a mutex for the timeout duration.

Once the timeout is reached,this will return None,
otherwise it will return the mutex guard.


# Example

```
use abi_stable::{
    external_types::RMutex,
    std_types::RDuration,
};

static MUTEX:RMutex<usize>=RMutex::new(0);

static DUR:RDuration=RDuration::from_millis(4);

let mut guard=MUTEX.try_lock_for(DUR).unwrap();

assert!( MUTEX.try_lock_for(DUR).is_none() );

assert_eq!(*guard,0);

```

*/
    #[inline]
    pub fn try_lock_for(&self, timeout: RDuration) -> ROption<RMutexGuard<'_,T>>{
        if self.vtable().try_lock_for()(&self.raw_mutex,timeout) {
            RSome(self.make_guard())
        }else{
            RNone
        }
    }
}

unsafe impl<T:Send> Send for RMutex<T>
where RawMutex:Send
{}

unsafe impl<T:Send> Sync for RMutex<T>
where RawMutex:Sync
{}

///////////////////////////////////////////////////////////////////////////////


impl<'a,T> Display for RMutexGuard<'a, T> 
where
    T:Display
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Display::fmt(&**self,f)
    }
}


impl<'a,T> Debug for RMutexGuard<'a, T> 
where
    T:Debug
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        Debug::fmt(&**self,f)
    }
}


impl<'a,T> Deref for RMutexGuard<'a, T> {
    type Target=T;

    fn deref(&self)->&T{
        unsafe{ &*self.rmutex.data.get() }
    }
}


impl<'a,T> DerefMut for RMutexGuard<'a, T> {
    fn deref_mut(&mut self)->&mut T{
        unsafe{ &mut *self.rmutex.data.get() }
    }
}

impl<'a,T> Drop for RMutexGuard<'a, T> {
    fn drop(&mut self){
        let vtable=self.rmutex.vtable();
        vtable.unlock()(&self.rmutex.raw_mutex);
    }
}



///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_struct="VTable")))]
#[sabi(missing_field(panic))]
struct VTableVal{
    lock:extern "C" fn(this:&RRawMutex),
    try_lock:extern "C" fn(this:&RRawMutex) -> bool,
    unlock:extern "C" fn(this:&RRawMutex),
    #[sabi(last_prefix_field)]
    try_lock_for:extern "C" fn(this:&RRawMutex, timeout: RDuration) -> bool,
}

impl VTable{
    // The VTABLE for this type in this executable/library
    const VTABLE: StaticRef<WithMetadata<VTableVal>> = {
        StaticRef::new(&WithMetadata::new(
            PrefixTypeTrait::METADATA,
            VTableVal{
                lock,
                try_lock,
                unlock,
                try_lock_for,
            }
        ))
    };
}


extern "C" fn lock(this:&RRawMutex){
    extern_fn_panic_handling!{
        this.inner.value.lock();
    }
}
extern "C" fn try_lock(this:&RRawMutex) -> bool{
    extern_fn_panic_handling!{
        this.inner.value.try_lock()
    }
}
extern "C" fn unlock(this:&RRawMutex){
    extern_fn_panic_handling!{
        this.inner.value.unlock();
    }
}
extern "C" fn try_lock_for(this:&RRawMutex, timeout: RDuration) -> bool{
    extern_fn_panic_handling!{
        this.inner.value.try_lock_for(timeout.into())
    }
}


///////////////////////////////////////////////////////////////////////////////




#[cfg(all(test,not(feature="only_new_tests")))]
mod tests{
    use super::*;

    use std::{
        thread,
        time::Duration,
    };

    use crossbeam_utils::thread::scope as scoped_thread;

    use crate::test_utils::check_formatting_equivalence;

    #[test]
    fn get_mut(){
        let mut mutex:RMutex<usize>=RMutex::new(0);
        *mutex.lock()+=100;
        *mutex.get_mut()+=100;
        *mutex.lock()+=100;
        assert_eq!(*mutex.lock(), 300);
    }


    #[test]
    fn into_inner(){
        let mutex:RMutex<usize>=RMutex::new(0);
        *mutex.lock()+=100;
        assert_eq!(mutex.into_inner(), 100);
    }

    #[test]
    fn debug_display(){
        let str_="\nhello\rhello\rhello\n";
        let mutex=RMutex::new(str_);
        let guard=mutex.lock();

        check_formatting_equivalence(&guard,str_);
    }

    #[test]
    fn lock(){
        static MUTEX:RMutex<usize>=RMutex::new(0);

        scoped_thread(|scope|{
            for _ in 0..8 {
                scope.spawn(move|_|{
                    for _ in 0..0x1000 {
                        *MUTEX.lock()+=1;
                    }
                });
            }
        }).unwrap();

        assert_eq!(*MUTEX.lock(),0x8000);
    }

    #[test]
    fn try_lock(){
        static MUTEX:RMutex<usize>=RMutex::new(0);

        scoped_thread(|scope|{
            for _ in 0..8 {
                scope.spawn(move|_|{
                    for _ in 0..0x1000 {
                        loop {
                            if let RSome(mut guard)=MUTEX.try_lock() {
                                *guard+=1;
                                break;
                            }
                        }
                    }
                });
            }
        }).unwrap();

        scoped_thread(|scope|{
            let _guard=MUTEX.lock();
            scope.spawn(move|_|{
                assert_eq!(MUTEX.try_lock().map(drop), RNone);
            });
            thread::sleep(Duration::from_millis(100));
        }).unwrap();

        assert_eq!(*MUTEX.lock(),0x8000);
    }

    #[test]
    fn try_lock_for(){
        static MUTEX:RMutex<usize>=RMutex::new(0);

        scoped_thread(|scope|{
            for _ in 0..8 {
                scope.spawn(move|_|{
                    for i in 0..0x1000 {
                        let wait_for=RDuration::new(0,(i+1)*500_000);
                        loop {
                            if let RSome(mut guard)=MUTEX.try_lock_for(wait_for) {
                                *guard+=1;
                                break;
                            }
                        }
                    }
                });
            }
        }).unwrap();


        scoped_thread(|scope|{
            let _guard=MUTEX.lock();
            scope.spawn(move|_|{
                assert_eq!(MUTEX.try_lock_for(RDuration::new(0,100_000)).map(drop), RNone);
            });
            thread::sleep(Duration::from_millis(100));
        }).unwrap();


        assert_eq!(*MUTEX.lock(),0x8000);
    }

}
