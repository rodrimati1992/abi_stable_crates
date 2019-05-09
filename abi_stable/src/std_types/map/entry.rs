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


#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    inside_abi_stable_crate,
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
#[sabi(inside_abi_stable_crate)]
struct ErasedOccupiedEntry<K,V>(PhantomData<Tuple2<K,V>>);

#[derive(StableAbi)]
#[repr(C)]
#[sabi(inside_abi_stable_crate)]
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
    pub fn get(&self) -> Option<&V> {
        match self {
            REntry::Occupied(entry) => Some(entry.get()),
            REntry::Vacant(entry) => None,
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut V> {
        match self {
            REntry::Occupied(entry) => Some(entry.get_mut()),
            REntry::Vacant(entry) => None,
        }
    }

    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            REntry::Occupied(entry) => entry.into_mut(),
            REntry::Vacant(entry) => entry.insert(default),
        }
    }

    pub fn or_insert_with<F>(self, default: F) -> &'a mut V 
    where 
        F: FnOnce() -> V
    {
        match self {
            REntry::Occupied(entry) => entry.into_mut(),
            REntry::Vacant(entry) => entry.insert(default()),
        }
    }

    pub fn key(&self) -> &K {
        match self {
            REntry::Occupied(entry) => entry.key(),
            REntry::Vacant(entry) => entry.key(),
        }
    }

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

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    inside_abi_stable_crate,
    bound="K:'a",
    bound="V:'a",
)]
pub struct ROccupiedEntry<'a,K,V>{
    entry:&'a mut ErasedOccupiedEntry<K,V>,
    vtable:*const OccupiedVTable<K,V>,
    _marker:UnsafeIgnoredType<OccupiedEntry<'a,K,V>>
}

#[derive(StableAbi)]
#[repr(C)]
#[sabi(
    inside_abi_stable_crate,
    bound="K:'a",
    bound="V:'a",
)]
pub struct RVacantEntry<'a,K,V>{
    entry:&'a mut ErasedVacantEntry<K,V>,
    vtable:*const VacantVTable<K,V>,
    _marker:UnsafeIgnoredType<VacantEntry<'a,K,V>>
}

/////////////////////////////////////////////////////////////////////////////////////////////

impl<'a,K,V> ROccupiedEntry<'a,K,V>{
    fn vtable<'b>(&self)->&'b OccupiedVTable<K,V>{
        unsafe{ &*self.vtable }
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
                vtable:(&*OccupiedVTable::VTABLE_REF).as_prefix_raw() ,
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    pub fn key(&self)->&K{
        let vtable=self.vtable();

        vtable.key()(&self.entry)
    }

    pub fn get(&self)->&V{
        let vtable=self.vtable();

        vtable.get_elem()(&self.entry)
    }

    pub fn get_mut(&mut self)->&mut V{
        let vtable=self.vtable();

        vtable.get_mut_elem()(&mut self.entry)
    }

    pub fn into_mut(self)->&'a mut V{
        let vtable=self.vtable();

        vtable.into_mut_elem()(self)
    }

    pub fn insert(&mut self,value:V)->V{
        let vtable=self.vtable();

        vtable.insert_elem()(&mut self.entry,value)
    }

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
    fn vtable<'b>(&self)->&'b VacantVTable<K,V>{
        unsafe{ &*self.vtable }
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
                vtable:(&*VacantVTable::VTABLE_REF).as_prefix_raw(),
                _marker:UnsafeIgnoredType::DEFAULT,
            }
        }
    }

    pub fn key(&self) -> &K {
        let vtable=self.vtable();

        vtable.key()(self.entry)
    }

    pub fn into_key(self) -> K {
        let vtable=self.vtable();

        vtable.into_key()(self)
    }

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
    inside_abi_stable_crate,
    kind(Prefix(prefix_struct="OccupiedVTable")),
    missing_field(panic),
)]
pub struct OccupiedVTableVal<K,V>{
    drop_entry:unsafe extern fn(&mut ErasedOccupiedEntry<K,V>),
    key:extern fn(&ErasedOccupiedEntry<K,V>)->&K,
    get_elem:extern fn(&ErasedOccupiedEntry<K,V>)->&V,
    get_mut_elem:extern fn(&mut ErasedOccupiedEntry<K,V>)->&mut V,
    into_mut_elem:extern fn(ROccupiedEntry<'_,K,V>)->&'_ mut V,
    insert_elem:extern fn(&mut ErasedOccupiedEntry<K,V>,V)->V,
    remove:extern fn(ROccupiedEntry<'_,K,V>)->V,
}


impl<K,V> OccupiedVTable<K,V>{
    const VTABLE_REF: *const WithMetadata<OccupiedVTableVal<K,V>> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
    };

    const VTABLE:OccupiedVTableVal<K,V>=OccupiedVTableVal{
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
    unsafe extern fn drop_entry(&mut self){
        extern_fn_panic_handling!{
            Self::run_as_unerased(self,|this|{
                ManuallyDrop::drop(this);
            }) 
        }
    }
    extern fn key(&self)->&K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.key().as_ref()
            )
        }}
    }
    extern fn get_elem(&self)->&V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.get() 
            )
        }}
    }
    extern fn get_mut_elem(&mut self)->&mut V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.get_mut() 
            )
        }}
    }
    extern fn into_mut_elem(this:ROccupiedEntry<'_,K,V>)->&'_ mut V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).into_mut()  
            )
        }}
    }
    extern fn insert_elem(&mut self,elem:V)->V{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.insert(elem) 
            )
        }}
    }
    extern fn remove(this:ROccupiedEntry<'_,K,V>)->V{
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
    inside_abi_stable_crate,
    kind(Prefix(prefix_struct="VacantVTable")),
    missing_field(panic),
)]
pub struct VacantVTableVal<K,V>{
    drop_entry:unsafe extern fn(&mut ErasedVacantEntry<K,V>),
    key:extern fn(&ErasedVacantEntry<K,V>)->&K,
    into_key:extern fn(RVacantEntry<'_,K,V>)->K,
    insert_elem:extern fn(RVacantEntry<'_,K,V>,V)->&'_ mut V,
}


impl<K,V> VacantVTable<K,V>{
    const VTABLE_REF: *const WithMetadata<VacantVTableVal<K,V>> = {
        &WithMetadata::new(PrefixTypeTrait::METADATA,Self::VTABLE)
    };

    const VTABLE:VacantVTableVal<K,V>=VacantVTableVal{
        drop_entry   :ErasedVacantEntry::drop_entry,
        key          :ErasedVacantEntry::key,
        into_key     :ErasedVacantEntry::into_key,
        insert_elem :ErasedVacantEntry::insert_elem,
    };
}


impl<K,V> ErasedVacantEntry<K,V>{
    unsafe extern fn drop_entry(&mut self){
        extern_fn_panic_handling!{
            Self::run_as_unerased(self,|this|{
                ManuallyDrop::drop(this);
            }) 
        }
    }
    extern fn key(&self)->&K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                self,
                |this| this.key().as_ref()
            ) 
        }}
    }
    extern fn into_key<'a>(this:RVacantEntry<'_,K,V>)->K{
        unsafe{extern_fn_panic_handling!{
            Self::run_as_unerased(
                this.into_inner(),
                |this| take_manuallydrop(this).into_key().into_inner()
            )
        }}
    }
    extern fn insert_elem(this:RVacantEntry<'_,K,V>,elem:V)->&'_ mut V{
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
