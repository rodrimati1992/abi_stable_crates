/*!
A late-initialized static reference.
*/

use std::{
    sync::atomic::{AtomicPtr,Ordering},
    ptr,
};

use crate::external_types::RMutex;

/**
A late-initialized static reference,with fallible initialization.

As opposed to `Once`,
this allows initialization of its static reference to happen fallibly,
by returning a `Result<_,_>` from the try_init function,
or by panicking inside either initialization function.

On `Err(_)` and panics,one can try initialializing the static reference again.

# Example

This lazily loads a configuration file.

```

use abi_stable::{
    sabi_types::LateStaticRef,
    std_types::{RBox,RBoxError,RHashMap,RString},
    utils::leak_value,
};

use std::{
    fs,
    io,
    path::Path,
};

use serde::Deserialize;


#[derive(Deserialize)]
pub struct Config{
    pub user_actions:RHashMap<RString,UserAction>,
}

#[derive(Deserialize)]
pub enum UserAction{
    Include,
    Ignore,
    ReplaceWith,
}


fn load_module(file_path:&Path)->Result<&'static Config,RBoxError>{
    static CONFIG:LateStaticRef<Config>=LateStaticRef::new();
    
    CONFIG.try_init(||{
        let file=load_file(file_path).map_err(RBoxError::new)?;
        let config=serde_json::from_str::<Config>(&file).map_err(RBoxError::new)?;
        Ok(leak_value(config))
    })
}


# fn load_file(file_path:&Path)->Result<String,RBoxError>{
#     let str=r##"
#         {
#             "user_actions":{
#                 "oolong":"prolonged",
#                 "genius":"idiot"
#             }
#         }
#     "##.to_string();
#     Ok(str)
# }

```

*/
#[repr(C)]
#[derive(StableAbi)]
pub struct LateStaticRef<T>{
    pointer:AtomicPtr<T>,
    lock:RMutex<()>,
}

const LOCK:RMutex<()>=RMutex::new(());


impl<T> LateStaticRef<T>{
    /// Constructs the late initialized static reference,
    /// in an uninitialized state.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::LateStaticRef;
    ///
    /// static LATE_REF:LateStaticRef<String>=LateStaticRef::new();
    ///
    /// ```
    pub const fn new()->Self{
        Self{
            lock:LOCK,
            pointer:AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Constructs the late initialized static reference,
    /// initialized with `value`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::sabi_types::LateStaticRef;
    ///
    /// static LATE_REF:LateStaticRef<&'static str>=
    ///     LateStaticRef::initialized(&"Hello!");
    ///
    /// ```
    pub const fn initialized(value:&'static T)->Self{
        Self{
            lock:LOCK,
            pointer:AtomicPtr::new(value as *const T as *mut T),
        }
    }

    /// Lazily initializes the reference with `initializer`,
    /// returning the reference if either it was already initialized,or
    /// if `initalizer` returned Ok(..).
    ///
    /// If `initializer` returns an `Err(...)` this returns the error and 
    /// allows the reference to be initializer later.
    ///
    /// If `initializer` panics,the panic is propagated,
    /// and the reference can be initalized later.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     sabi_types::LateStaticRef,
    ///     utils::leak_value,
    /// };
    ///
    /// static LATE:LateStaticRef<String>=LateStaticRef::new();
    ///
    /// static EARLY:LateStaticRef<&'static str>=
    ///     LateStaticRef::initialized(&"Hello!");
    ///
    /// assert_eq!( LATE.try_init(|| Err("oh no!") ), Err("oh no!") );
    /// assert_eq!( 
    ///     LATE
    ///         .try_init(||->Result<&'static String,()>{
    ///             Ok( leak_value("Yay".to_string()) )
    ///         })
    ///         .map(|s| s.as_str() ),
    ///     Ok("Yay"),
    /// );
    /// 
    /// assert_eq!( EARLY.try_init(|| Err("oh no!") ), Ok(&"Hello!") );
    ///
    ///
    /// ```
    pub fn try_init<F,E>(&self,initializer:F)->Result<&'static T,E>
    where F:FnOnce()->Result<&'static T,E>
    {
        if let Some(pointer)=self.get() {
            return Ok(pointer);
        }
        
        let guard_=self.lock.lock();
        
        if let Some(pointer)=self.get() {
            return Ok(pointer);
        }

        let pointer=initializer()?;

        self.pointer.store(pointer as *const T as *mut T,Ordering::Release);

        drop(guard_);

        Ok(pointer)

    }


    /// Lazily initializes the reference with `initializer`,
    /// returning the reference if either it was already initialized,or
    /// once `initalizer` returns the reference.
    ///
    /// If `initializer` panics,the panic is propagated,
    /// and the reference can be initalized later.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     sabi_types::LateStaticRef,
    ///     utils::leak_value,
    /// };
    ///
    /// static LATE:LateStaticRef<String>=LateStaticRef::new();
    ///
    /// static EARLY:LateStaticRef<&'static str>=
    ///     LateStaticRef::initialized(&"Hello!");
    ///
    /// let _=std::panic::catch_unwind(||{
    ///     LATE.init(|| panic!() );
    /// });
    ///
    /// assert_eq!( LATE.init(|| leak_value("Yay".to_string()) ), &"Yay" );
    ///
    /// assert_eq!( EARLY.init(|| panic!() ), &"Hello!" );
    ///
    /// ```
    #[inline]
    pub fn init<F>(&self,initializer:F)->&'static T
    where F:FnOnce()->&'static T
    {
        self
            .try_init(||->Result<&'static T,()>{ 
                Ok(initializer()) 
            })
            .expect("bug:LateStaticRef::try_init should only return an Err if `initializer` does")
    }

    /// Returns `Some(x:&'static T)` if the reference was initialized,otherwise returns None.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::{
    ///     sabi_types::LateStaticRef,
    ///     utils::leak_value,
    /// };
    ///
    /// static LATE:LateStaticRef<String>=LateStaticRef::new();
    ///
    /// static EARLY:LateStaticRef<&'static str>=
    ///     LateStaticRef::initialized(&"Hello!");
    ///
    /// let _=std::panic::catch_unwind(||{
    ///     LATE.init(|| panic!() );
    /// });
    ///
    /// assert_eq!( LATE.get(), None );
    /// LATE.init(|| leak_value("Yay".to_string()) );
    /// assert_eq!( LATE.get().map(|s| s.as_str() ), Some("Yay") );
    ///
    /// assert_eq!( EARLY.get(), Some(&"Hello!") );
    ///
    /// ```
    pub fn get(&self)->Option<&'static T>{
        unsafe{
            self.pointer
                .load(Ordering::Acquire)
                .as_ref()
        }
    }
}

use ::std::panic::{
    UnwindSafe,
    RefUnwindSafe,
};

impl<T> UnwindSafe for LateStaticRef<T>{}
impl<T> RefUnwindSafe for LateStaticRef<T>{}


//////////////////////////////////////////////////////


//#[cfg(test)]
#[cfg(all(test,not(feature="only_new_tests")))]
mod tests{
    use super::*;

    use std::panic::catch_unwind;

    static N_100:u32=100;
    static N_277:u32=277;

    #[test]
    fn test_init(){
        let ptr=LateStaticRef::<u32>::new();

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
        let ptr=LateStaticRef::<u32>::new();

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