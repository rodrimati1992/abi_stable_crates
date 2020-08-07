use super::*;

use std::{
    collections::hash_map::{OccupiedEntry,VacantEntry,Entry},
    mem::ManuallyDrop,
    ptr,
};

use crate::{
    marker_type::UnsafeIgnoredType,
    prefix_type::{WithMetadata,PrefixTypeTrait},
};


/// The enum stored alongside the unerased HashMap.
pub(super) enum BoxedREntry<'a,K,V>{
    Occupied(UnerasedOccupiedEntry<'a,K,V>),
    Vacant(UnerasedVacantEntry<'a,K,V>),
}

/// A handle into an entry in a map, which is either vacant or occupied.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound="K:'a",
    bound="V:'a",
)]
pub enum REntry<'a,K,V>{
    Occupied(ROccupiedEntry<'a,K,V>),
    Vacant(RVacantEntry<'a,K,V>),
}


/////////////////////////////////////////////////////////////////////////////////////////////


#[derive(StableAbi)]
#[repr(C)]
struct ErasedOccupiedEntry<K,V>(PhantomData<Tuple2<K,V>>);

#[derive(StableAbi)]
#[repr(C)]
struct ErasedVacantEntry  <K,V>(PhantomData<Tuple2<K,V>>);

type UnerasedOccupiedEntry<'a,K,V>=
    ManuallyDrop<OccupiedEntry<'a,MapKey<K>,V>>;

type UnerasedVacantEntry<'a,K,V>=
    ManuallyDrop<VacantEntry<'a,MapKey<K>,V>>;


unsafe impl<'a,K:'a,V:'a> ErasedType<'a> for ErasedOccupiedEntry<K,V> {
    type Unerased=UnerasedOccupiedEntry<'a,K,V>;
}

unsafe impl<'a,K:'a,V:'a> ErasedType<'a> for ErasedVacantEntry<K,V> {
    type Unerased=UnerasedVacantEntry<'a,K,V>;
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a,K,V> From<Entry<'a,MapKey<K>,V>> for BoxedREntry<'a,K,V>
where
    K:Eq+Hash
{
    fn from(entry:Entry<'a,MapKey<K>,V>)->Self{
        match entry {
            Entry::Occupied(entry)=>
                entry.piped(ManuallyDrop::new).piped(BoxedREntry::Occupied),
            Entry::Vacant(entry)  =>
                entry.piped(ManuallyDrop::new).piped(BoxedREntry::Vacant),
        }
    }
}

impl<'a,K,V> REntry<'a,K,V>
where
    K:Eq+Hash
{
    pub(super)unsafe fn new(entry:&'a mut BoxedREntry<'a,K,V>)->Self{
        match entry {
            BoxedREntry::Occupied(entry)=>
                entry.piped(ROccupiedEntry::new).piped(REntry::Occupied),
            BoxedREntry::Vacant(entry)  =>
                entry.piped(RVacantEntry::new).piped(REntry::Vacant),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////



impl<'a, K, V> REntry<'a, K, V> {
    /// Returns a reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map:RHashMap<u32,u32>=vec![(1,100)].into_iter().collect();
    ///
    /// assert_eq!(map.entry(0).get(),None);
    /// assert_eq!(map.entry(1).get(),Some(&100));
    ///
    /// ```
    pub fn get(&self) -> Option<&V> {
        match self {
            REntry::Occupied(entry) => Some(entry.get()),
            REntry::Vacant(_) => None,
        }
    }

    /// Returns a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map:RHashMap<u32,u32>=vec![(1,100)].into_iter().collect();
    ///
    /// assert_eq!(map.entry(0).get_mut(),None);
    /// assert_eq!(map.entry(1).get_mut(),Some(&mut 100));
    ///
    /// ```
    pub fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            REntry::Occupied(entry) => Some(entry.get_mut()),
            REntry::Vacant(_) => None,
        }
    }

    /// Inserts `default` as the value in the entry if it wasn't occupied,
    /// returning a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// assert_eq!(map.entry(0).or_insert(100),&mut 100);
    ///
    /// assert_eq!(map.entry(0).or_insert(400),&mut 100);
    ///
    /// ```
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            REntry::Occupied(entry) => entry.into_mut(),
            REntry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Inserts `default()` as the value in the entry if it wasn't occupied,
    /// returning a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap,RString};
    ///
    /// let mut map=RHashMap::<u32,RString>::new();
    ///
    /// assert_eq!(
    ///     map.entry(0).or_insert_with(|| "foo".into() ),
    ///     &mut RString::from("foo")
    /// );
    ///
    /// assert_eq!(
    ///     map.entry(0).or_insert_with(|| "bar".into() ),
    ///     &mut RString::from("foo")
    /// );
    ///
    /// ```
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V 
    where 
        F: FnOnce() -> V
    {
        match self {
            REntry::Occupied(entry) => entry.into_mut(),
            REntry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Gets the key of the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap,RString};
    ///
    /// let mut map=RHashMap::<RString,RString>::new();
    /// map.insert("foo".into(),"bar".into());
    ///
    /// assert_eq!(
    ///     map.entry("foo".into()).key(),
    ///     &RString::from("foo")
    /// );
    /// ```
    pub fn key(&self) -> &K {
        match self {
            REntry::Occupied(entry) => entry.key(),
            REntry::Vacant(entry) => entry.key(),
        }
    }

    /// Allows mutating an occupied entry before doing other operations.
    /// 
    /// This is a no-op on a vacant entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::{RHashMap,RString};
    ///
    /// let mut map=RHashMap::<RString,RString>::new();
    /// map.insert("foo".into(),"bar".into());
    ///
    /// assert_eq!(
    ///     map.entry("foo".into())
    ///        .and_modify(|x| x.push_str("hoo") )
    ///        .get(),
    ///     Some(&RString::from("barhoo"))
    /// );
    /// ```
    pub fn and_modify<F>(self, f: F) -> Self
    where 
        F: FnOnce(&mut V)
    {
        match self {
            REntry::Occupied(mut entry) => {
                f(entry.get_mut());
                REntry::Occupied(entry)
            },
            REntry::Vacant(entry) => REntry::Vacant(entry),
        }
    }

    /// Inserts the `V::default()` value in the entry if it wasn't occupied,
    /// returning a mutable reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::RHashMap;
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// assert_eq!(map.entry(0).or_insert(100),&mut 100);
    /// assert_eq!(map.entry(0).or_default(),&mut 100);
    ///
    /// assert_eq!(map.entry(1).or_default(),&mut 0);
    ///
    /// ```
    pub fn or_default(self) -> &'a mut V 
    where
        V: Default
    {
        match self {
            REntry::Occupied(entry) => entry.into_mut(),
            REntry::Vacant(entry) => entry.insert(Default::default()),
        }
    }
}


impl<K,V> Debug for REntry<'_,K,V>
where
    K:Debug,
    V:Debug,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        match self {
            REntry::Occupied(entry)=>Debug::fmt(entry,f),
            REntry::Vacant(entry)=>Debug::fmt(entry,f),            
        }
    }
}


/////////////////////////////////////////////////////////////////////////////////////////////

/// A handle into an occupied entry in a map,always a variant of an REntry.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound="K:'a",
    bound="V:'a",
)]
pub struct ROccupiedEntry<'a,K,V>{
    entry:&'a mut ErasedOccupiedEntry<K,V>,
    vtable:OccupiedVTable_Ref<K,V>,
    _marker:UnsafeIgnoredType<OccupiedEntry<'a,K,V>>
}

/// A handle into a vacant entry in a map,always a variant of an REntry.
#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    bound="K:'a",
    bound="V:'a",
)]
pub struct RVacantEntry<'a,K,V>{
    entry:&'a mut ErasedVacantEntry<K,V>,
    vtable:VacantVTable_Ref<K,V>,
    _marker:UnsafeIgnoredType<VacantEntry<'a,K,V>>
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a,K,V> ROccupiedEntry<'a,K,V>{
    fn vtable(&self)->OccupiedVTable_Ref<K,V>{
        self.vtable
    }
}

impl<'a,K,V> ROccupiedEntry<'a,K,V>{
    fn into_inner(self)->&'a mut ErasedOccupiedEntry<K,V>{
        let mut this=ManuallyDrop::new(self);
        unsafe{ ((&mut this.entry) as *mut &'a mut ErasedOccupiedEntry<K,V>).read() }
    }

    pub(super) fn new(entry:&'a mut UnerasedOccupiedEntry<'a,K,V>)->Self{
        unsafe{ 
            Self{
                entry:ErasedOccupiedEntry::from_unerased(entry),
                vtable:OccupiedVTable::VTABLE_REF,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    /// Gets the key of the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// map.insert(0,100);
    ///
    /// match map.entry(0) {
    ///     REntry::Occupied(entry)=>{
    ///         assert_eq!(entry.key(),&0);
    ///     }
    ///     REntry::Vacant(_)=>unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn key(&self)->&K{
        let vtable=self.vtable();

        vtable.key()(&self.entry)
    }

    /// Gets a reference to the value in the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// map.insert(6,15);
    ///
    /// match map.entry(6) {
    ///     REntry::Occupied(entry)=>{
    ///         assert_eq!(entry.get(),&15);
    ///     }
    ///     REntry::Vacant(_)=>unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn get(&self)->&V{
        let vtable=self.vtable();

        vtable.get_elem()(&self.entry)
    }

    /// Gets a mutable reference to the value in the entry.
    /// To borrow with the lifetime of the map,use `ROccupiedEntry::into_mut`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// map.insert(6,15);
    ///
    /// match map.entry(6) {
    ///     REntry::Occupied(mut entry)=>{
    ///         assert_eq!(entry.get_mut(),&mut 15);
    ///     }
    ///     REntry::Vacant(_)=>unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn get_mut(&mut self)->&mut V{
        let vtable=self.vtable();

        vtable.get_mut_elem()(&mut self.entry)
    }

    /// Gets a mutable reference to the value in the entry,
    /// that borrows with the lifetime of the map instead of from this `ROccupiedEntry`.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<String,u32>::new();
    ///
    /// map.insert("baz".into(),0xDEAD);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(entry)=>{
    ///         assert_eq!(entry.into_mut(),&mut 0xDEAD);
    ///     }
    ///     REntry::Vacant(_)=>unreachable!(),
    /// };
    ///
    ///
    /// ```
    pub fn into_mut(self)->&'a mut V{
        let vtable=self.vtable();

        vtable.into_mut_elem()(self)
    }

    /// Replaces the current value of the entry with `value`,returning the previous value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<String,u32>::new();
    ///
    /// map.insert("baz".into(),0xD00D);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(mut entry)=>{
    ///         assert_eq!(entry.insert(0xDEAD),0xD00D);
    ///     }
    ///     REntry::Vacant(_)=>{
    ///         unreachable!();
    ///     }
    /// }
    ///
    /// assert_eq!( map.get("baz"), Some(&0xDEAD));
    ///
    /// ```
    pub fn insert(&mut self,value:V)->V{
        let vtable=self.vtable();

        vtable.insert_elem()(&mut self.entry,value)
    }

    /// Removes the entry from the map,returns the value.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<String,u32>::new();
    ///
    /// map.insert("baz".into(),0xDEAD);
    ///
    /// match map.entry("baz".into()) {
    ///     REntry::Occupied(entry)=>{
    ///         assert_eq!(entry.remove(),0xDEAD);
    ///     }
    ///     REntry::Vacant(_)=>{
    ///         unreachable!();
    ///     }
    /// }
    ///
    /// assert!( ! map.contains_key("baz") );
    ///
    /// ```
    pub fn remove(self)->V{
        let vtable=self.vtable();

        vtable.remove()(self)
    }
}


impl<K,V> Debug for ROccupiedEntry<'_,K,V>
where
    K:Debug,
    V:Debug,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("ROccupiedEntry")
         .field("key",self.key())
         .field("value",self.get())
         .finish()
    }
}


impl<'a,K,V> Drop for ROccupiedEntry<'a,K,V>{
    fn drop(&mut self){
        let vtable=self.vtable();

        unsafe{
            vtable.drop_entry()(self.entry);
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a,K,V> RVacantEntry<'a,K,V>{
    fn vtable(&self)->VacantVTable_Ref<K,V>{
        self.vtable
    }
}

impl<'a,K,V> RVacantEntry<'a,K,V>{
    fn into_inner(self)->&'a mut ErasedVacantEntry<K,V>{
        let mut this=ManuallyDrop::new(self);
        unsafe{ ((&mut this.entry) as *mut &'a mut ErasedVacantEntry<K,V>).read() }
    }

    pub(super) fn new(entry:&'a mut UnerasedVacantEntry<'a,K,V>)->Self
    where
        K:Eq+Hash
    {
        unsafe{
            Self{
                entry:ErasedVacantEntry::from_unerased(entry),
                vtable: VacantVTable::VTABLE_REF,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    /// Gets the key of the entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<u32,u32>::new();
    ///
    /// match map.entry(1337) {
    ///     REntry::Occupied(_)=>{
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry)=>{
    ///         assert_eq!(entry.key(),&1337);
    ///     }
    /// }
    ///
    /// assert_eq!( map.get(&1337), None );
    ///
    /// ```
    pub fn key(&self) -> &K {
        let vtable=self.vtable();

        vtable.key()(self.entry)
    }

    /// Gets back the key that was passed to RHashMap::entry.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<String,u32>::new();
    ///
    /// match map.entry("lol".into()) {
    ///     REntry::Occupied(_)=>{
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry)=>{
    ///         assert_eq!(entry.into_key(),"lol".to_string());
    ///     }
    /// }
    ///
    /// assert_eq!( map.get("lol"), None );
    ///
    /// ```
    pub fn into_key(self) -> K {
        let vtable=self.vtable();

        vtable.into_key()(self)
    }

    /// Sets the value of the entry,returning a mutable reference to it.
    ///
    /// # Example
    ///
    /// ```
    /// use abi_stable::std_types::map::{REntry,RHashMap};
    ///
    /// let mut map=RHashMap::<String,u32>::new();
    ///
    /// match map.entry("lol".into()) {
    ///     REntry::Occupied(_)=>{
    ///         unreachable!();
    ///     }
    ///     REntry::Vacant(entry)=>{
    ///         assert_eq!(entry.insert(67),&mut 67);
    ///     }
    /// }
    ///
    /// assert_eq!( map.get("lol"), Some(&67) );
    ///
    /// ```
    pub fn insert(self, value: V) -> &'a mut V {
        let vtable=self.vtable();

        vtable.insert_elem()(self,value)
    }
}



impl<K,V> Debug for RVacantEntry<'_,K,V>
where
    K:Debug,
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("RVacantEntry")
         .field("key",self.key())
         .finish()
    }
}


impl<'a,K,V> Drop for RVacantEntry<'a,K,V>{
    fn drop(&mut self){
        let vtable=self.vtable();

        unsafe{
            vtable.drop_entry()(self.entry)
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////



#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
)]
pub struct OccupiedVTable<K,V>{
    drop_entry:unsafe extern "C" fn(&mut ErasedOccupiedEntry<K,V>),
    key:extern "C" fn(&ErasedOccupiedEntry<K,V>)->&K,
    get_elem:extern "C" fn(&ErasedOccupiedEntry<K,V>)->&V,
    get_mut_elem:extern "C" fn(&mut ErasedOccupiedEntry<K,V>)->&mut V,
    into_mut_elem:extern "C" fn(ROccupiedEntry<'_,K,V>)->&'_ mut V,
    insert_elem:extern "C" fn(&mut ErasedOccupiedEntry<K,V>,V)->V,
    remove:extern "C" fn(ROccupiedEntry<'_,K,V>)->V,
}


impl<K,V> OccupiedVTable<K,V>{
    const VTABLE_REF: OccupiedVTable_Ref<K,V> = unsafe{
        OccupiedVTable_Ref(
            WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
                .as_prefix()
        )
    };

    const VTABLE:OccupiedVTable<K,V>=OccupiedVTable{
        drop_entry   :ErasedOccupiedEntry::drop_entry,
        key          :ErasedOccupiedEntry::key,
        get_elem     :ErasedOccupiedEntry::get_elem,
        get_mut_elem :ErasedOccupiedEntry::get_mut_elem,
        into_mut_elem:ErasedOccupiedEntry::into_mut_elem,
        insert_elem  :ErasedOccupiedEntry::insert_elem,
        remove       :ErasedOccupiedEntry::remove,
    };
}


impl<K,V> ErasedOccupiedEntry<K,V>{
    unsafe extern "C" fn drop_entry(&mut self){
        extern_fn_panic_handling!{
            Self::run_as_unerased(self,|this|{
                ManuallyDrop::drop(this);
            }) 
        }
    }
    extern "C" fn key(&self)->&K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.key().as_ref()
            )
        }}
    }
    extern "C" fn get_elem(&self)->&V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.get() 
            )
        }}
    }
    extern "C" fn get_mut_elem(&mut self)->&mut V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.get_mut() 
            )
        }}
    }
    extern "C" fn into_mut_elem(this:ROccupiedEntry<'_,K,V>)->&'_ mut V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).into_mut()  
            )
        }}
    }
    extern "C" fn insert_elem(&mut self,elem:V)->V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.insert(elem) 
            )
        }}
    }
    extern "C" fn remove(this:ROccupiedEntry<'_,K,V>)->V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).remove()  
            )
        }}
    }    
}





/////////////////////////////////////////////////////////////////////////////////////////////





#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    kind(Prefix),
    missing_field(panic),
)]
pub struct VacantVTable<K,V>{
    drop_entry:unsafe extern "C" fn(&mut ErasedVacantEntry<K,V>),
    key:extern "C" fn(&ErasedVacantEntry<K,V>)->&K,
    into_key:extern "C" fn(RVacantEntry<'_,K,V>)->K,
    insert_elem:extern "C" fn(RVacantEntry<'_,K,V>,V)->&'_ mut V,
}


impl<K,V> VacantVTable<K,V>{
    const VTABLE_REF: VacantVTable_Ref<K,V> = unsafe{
        VacantVTable_Ref(
            WithMetadata::new(
                PrefixTypeTrait::METADATA,
                Self::VTABLE,
            ).as_prefix()
        )
    };

    const VTABLE:VacantVTable<K,V>=VacantVTable{
        drop_entry   :ErasedVacantEntry::drop_entry,
        key          :ErasedVacantEntry::key,
        into_key     :ErasedVacantEntry::into_key,
        insert_elem :ErasedVacantEntry::insert_elem,
    };
}


impl<K,V> ErasedVacantEntry<K,V>{
    unsafe extern "C" fn drop_entry(&mut self){
        extern_fn_panic_handling!{
            Self::run_as_unerased(self,|this|{
                ManuallyDrop::drop(this);
            }) 
        }
    }
    extern "C" fn key(&self)->&K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.key().as_ref()
            ) 
        }}
    }
    extern "C" fn into_key<'a>(this:RVacantEntry<'_,K,V>)->K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).into_key().into_inner()
            )
        }}
    }
    extern "C" fn insert_elem(this:RVacantEntry<'_,K,V>,elem:V)->&'_ mut V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).insert(elem) 
            ) 
        }}
    }    
}




/////////////////////////////////////////////////////////////////////////////////////////////



/// Copy paste of the unstable `ManuallyDrop::take`
unsafe fn take_manuallydrop<T>(slot: &mut ManuallyDrop<T>) -> T {
    ManuallyDrop::into_inner(ptr::read(slot))
}
