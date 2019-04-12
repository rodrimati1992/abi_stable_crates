/*!
A late-initialized static reference.
*/

use std::{
    sync::atomic::{AtomicPtr,Ordering},
    ptr,
};

use lock_api::RawMutex as RawMutexTrait;

use parking_lot::RawMutex;

/// A late-initialized static reference.
pub struct LazyStaticRef<T>{
    // Using a RawMutex because all I need is to prevent multiple threads 
    // from loading the static at the same time.
    lock:RawMutex,
    pointer:AtomicPtr<T>,
}

// Workaround for being unable to use traits inside `const fn`.
const LOCK:RawMutex=<RawMutex as RawMutexTrait>::INIT;


impl<T> LazyStaticRef<T>{
    pub const fn new()->Self{
        Self{
            lock:LOCK,
            pointer:AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Attempts to initialize the `&'static T` reference this contains,
    /// if it is not already initialized.
    ///
    /// If `initializer` returns an `Err(...)` this returns the error and 
    /// allows the reference to be initializer later.
    ///
    /// If `initializer` panics this does not get poisoned,it can just be called again.
    ///
    /// If this is already initialized,`initializer` won't be run.
    pub fn try_init<F,E>(&self,initializer:F)->Result<&'static T,E>
    where F:FnOnce()->Result<&'static T,E>
    {
        self.lock.lock();
        let unlocker=UnlockOnDrop(&self.lock);

        if let Some(pointer)=self.get() {
            return Ok(pointer);
        }

        let pointer=initializer()?;

        self.pointer.store(pointer as *const T as *mut T,Ordering::Release);

        drop(unlocker);

        Ok(pointer)

    }


    /// Attempts to initialize the `&'static T` reference this contains,
    /// if it is not already initialized.
    ///
    /// If `initializer` panics this does not get poisoned,it can just be called again.
    ///
    /// If this is already initialized,`initializer` won't be run.
    #[inline]
    pub fn init<F>(&self,initializer:F)->&'static T
    where F:FnOnce()->&'static T
    {
        self
            .try_init(||->Result<&'static T,()>{ 
                Ok(initializer()) 
            })
            .expect("bug:LazyStaticRef::try_init should only return an Err if `initializer` does")
    }

    /// Returns `Some(x:&'static T)` if it was initialized,otherwise returns None.
    pub fn get(&self)->Option<&'static T>{
        unsafe{
            self.pointer
                .load(Ordering::Acquire)
                .as_ref()
        }
    }
}



struct UnlockOnDrop<'a>(&'a RawMutex);

impl<'a> Drop for UnlockOnDrop<'a>{
    fn drop(&mut self){
        (self.0).unlock();
    }
}


//////////////////////////////////////////////////////


#[cfg(test)]
mod tests{
    use super::*;

    use std::panic::catch_unwind;

    static N_100:u32=100;
    static N_277:u32=277;

    #[test]
    fn test_init(){
        let ptr=LazyStaticRef::<u32>::new();

        assert_eq!(None,ptr.get() );
        
        let caught=catch_unwind(||{
            ptr.init(|| panic!() );
        });
        assert!(caught.is_err());

        assert_eq!(None,ptr.get() );

        assert_eq!(100,*ptr.init(|| &N_100 ));
        assert_eq!(100,*ptr.init(|| panic!("this should not run") ));
        
        assert_eq!(
            (&N_100)as *const u32,
            ptr.get().unwrap() as *const u32 
        );

    }

    #[test]
    fn test_try_init(){
        let ptr=LazyStaticRef::<u32>::new();

        assert_eq!(None,ptr.get() );
        
        let caught=catch_unwind(||{
            let _=ptr.try_init(||->Result<_,i32>{ panic!() });
        });
        assert!(caught.is_err());

        assert_eq!(None,ptr.get() );

        assert_eq!(Err(10),ptr.try_init(||->Result<_,i32>{ Err(10) }));
        assert_eq!(Err(17),ptr.try_init(||->Result<_,i32>{ Err(17) }));

        assert_eq!(Ok(&277),ptr.try_init(||->Result<_,i32>{ Ok(&N_277) }));
        
        assert_eq!(
            Ok(&277),
            ptr.try_init(||->Result<_,i32>{ panic!("this should not run") })
        );
        
        assert_eq!(
            (&N_277)as *const u32,
            ptr.get().unwrap() as *const u32 
        );

    }
}