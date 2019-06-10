use super::*;

use std::{
    slice,
    cmp::Ordering,
};

/// The layout of a field.
#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi)]
pub struct TLFields {
    /// The field names,separating fields with ";".
    pub names: StaticStr,

    pub variant_lengths:StaticSlice<u16>,

    /// Which lifetimes in the struct are referenced in the field type.
    pub lifetime_indices: SliceAndFieldIndices<LifetimeIndex>,

    /// All the function pointer types in the field.
    pub functions:StaticSlice<WithFieldIndex<TLFunction>>,

    /// All TLField fields which map 1:1.
    pub field_1to1:StaticSlice<Field1to1>,
}


impl TLFields{
    pub const fn new(
        names: &'static str,
        variant_lengths:&'static [u16],
        lifetime_indices: SliceAndFieldIndices<LifetimeIndex>,
        functions:&'static [WithFieldIndex<TLFunction>],
        field_1to1:&'static [Field1to1],
    )->Self{
        Self{
            names:StaticStr::new(names),
            variant_lengths:StaticSlice::new(variant_lengths),
            lifetime_indices,
            functions:StaticSlice::new(functions),
            field_1to1:StaticSlice::new(field_1to1),
        }
    }

    pub fn get_fields(&self)->TLFieldsIterator{
        TLFieldsIterator{
            field_names:self.names.as_str().split(FIELD_SPLITTER),
            lifetime_indices:self.lifetime_indices.iter(),
            functions:SplitFieldBoundaries::new(
                self.functions.as_slice(),
                self.variant_lengths.as_slice(),
            ),
            mapped_1to1:self.field_1to1.as_slice().iter(),
        }
    }
    pub fn get_field_vec(&self)->Vec<TLField>{
        self.get_fields().collect()
    }
}


impl Eq for TLFields{}
impl PartialEq for TLFields{
    fn eq(&self,other:&Self)->bool{
        self.get_fields().eq(other.get_fields())
    }
}


///////////////////////////////////////////////////////////////////////////////

#[repr(u8)]
#[derive(Copy, Clone,Debug, StableAbi,PartialEq,Eq)]
pub enum TLFieldsOrSlice{
    TLFields(TLFields),
    Slice(StaticSlice<TLField>),
}

impl TLFieldsOrSlice{
    pub const fn from_slice(slice:&'static [TLField])->Self{
        TLFieldsOrSlice::Slice(StaticSlice::new(slice))
    }

    pub fn get_fields(&self)->TLFOSIter{
        match self {
            TLFieldsOrSlice::TLFields(x)=>TLFOSIter::TLFields(x.get_fields()),
            TLFieldsOrSlice::Slice(x)=>TLFOSIter::Slice(x.as_slice().iter()),
        }
    }
    pub fn get_field_vec(&self)->Vec<TLField>{
        self.get_fields().collect()
    }
    pub fn len(&self)->usize{
        match self {
            TLFieldsOrSlice::TLFields(x)=>x.field_1to1.len(),
            TLFieldsOrSlice::Slice(x)=>x.len(),
        }
    }
}


#[repr(C)]
#[derive(Clone,Debug)]
pub enum TLFOSIter{
    TLFields(TLFieldsIterator),
    Slice(slice::Iter<'static,TLField>),
}



impl Iterator for TLFOSIter{
    type Item=TLField;

    fn next(&mut self)->Option<TLField>{
        match self {
            TLFOSIter::TLFields(iter)=>iter.next(),
            TLFOSIter::Slice(iter)=>iter.next().cloned(),
        }
    }

    fn size_hint(&self)->(usize,Option<usize>){
        match self {
            TLFOSIter::TLFields(iter)=>iter.size_hint(),
            TLFOSIter::Slice(iter)=>iter.size_hint(),
        }
    }
    fn count(self) -> usize {
        match self {
            TLFOSIter::TLFields(iter)=>iter.count(),
            TLFOSIter::Slice(iter)=>iter.count(),
        }
    }
}


impl std::iter::ExactSizeIterator for TLFOSIter{}



///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct SliceAndFieldIndices<T:'static>{
    pub values: StaticSlice<T>,
    pub field_indices: StaticSlice<WithFieldIndex<usize>>,
}


impl<T> SliceAndFieldIndices<T>{
    pub const fn new(
        values: &'static [T],
        field_indices:&'static [WithFieldIndex<usize>],
    )->Self{
        Self{
            values:StaticSlice::new(values),
            field_indices:StaticSlice::new(field_indices),
        }
    }

    pub fn iter(&self)->SAFIIter<T>{
        SAFIIter{
            values:self.values.as_slice(),
            field_indices:self.field_indices.as_slice(),
        }
    }
}


#[repr(C)]
#[derive(Copy, Clone,Debug)]
pub struct SAFIIter<T:'static>{
    values:&'static [T],
    field_indices:&'static [WithFieldIndex<usize>],
}

impl<T:'static> Iterator for SAFIIter<T>{
    type Item=WithFieldIndex<&'static [T]>;

    fn next(&mut self)->Option<Self::Item>{
        let field_index=self.field_indices.get(0)?;
        let len=self.values.len();
        self.field_indices=&self.field_indices[1..];
        let next_ind=self.field_indices.first().map_or(len,|x| x.value );
        Some(WithFieldIndex{
            index:field_index.index,
            value:&self.values[field_index.value..next_ind],
        })
    }

    fn size_hint(&self)->(usize,Option<usize>){
        let len=self.values.len();
        (len,Some(len))
    }
    fn count(self) -> usize {
        self.values.len()
    }
}


impl<T> std::iter::ExactSizeIterator for SAFIIter<T>{}






///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct FieldIndex{
    pub variant:u16,
    pub field_pos:u16,
}

impl FieldIndex {
    pub const fn from_variant_field(variant:u16,field_pos:u16)->Self{
        Self{variant,field_pos}
    }
    pub fn increment(&mut self,variant_lengths:&[u16]){
        let next_field_pos=self.field_pos+1;
        if variant_lengths[self.variant as usize]as u16 == next_field_pos {
            let next_variant=self.variant+1;
            if variant_lengths.len()as u16 != next_variant {
                self.variant=next_variant;
            }
        }else{
            self.field_pos=next_field_pos;
        }
    }
}


///////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi)]
pub struct Field1to1{
    /// The layout of the field's type.
    ///
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static AbiInfo`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    pub abi_info: GetAbiInfo,

    /// Whether this field is only a function pointer.
    pub is_function:bool,

    pub field_accessor:FieldAccessor,
}


impl Field1to1{
    pub const fn new(
        abi_info: GetAbiInfo,
        is_function:bool,
        field_accessor:FieldAccessor,
    )->Self{
        Self{abi_info,is_function,field_accessor}
    }
}


///////////////////////////////////////////////////////////////////////////////


#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct WithFieldIndex<T>{
    pub index:FieldIndex,
    pub value:T,
}

impl<T> WithFieldIndex<T>{
    pub const fn from_vari_field_val(variant:u16,field_pos:u16,value:T)->Self{
        Self{
            index:FieldIndex{variant,field_pos},
            value,
        }
    }    
}


///////////////////////////////////////////////////////////////////////////////


const FIELD_SPLITTER:&'static [char]=&[';','|'];


#[derive(Clone,Debug)]
pub struct TLFieldsIterator{
    field_names:std::str::Split<'static,&'static [char]>,

    lifetime_indices:SAFIIter<LifetimeIndex>,
    functions:SplitFieldBoundaries<'static,TLFunction>,

    mapped_1to1:slice::Iter<'static,Field1to1>,
}


impl Iterator for TLFieldsIterator{
    type Item=TLField;

    fn next(&mut self)->Option<TLField>{
        let field_1to1=self.mapped_1to1.next()?.clone();

        Some(TLField{
            name:StaticStr::new(self.field_names.next().unwrap()),
            lifetime_indices:StaticSlice::new(self.lifetime_indices.next().unwrap().value),
            functions:StaticSlice::new(self.functions.next().unwrap()),
            abi_info:field_1to1.abi_info,
            is_function:field_1to1.is_function,
            field_accessor:field_1to1.field_accessor,
        })
    }

    fn size_hint(&self)->(usize,Option<usize>){
        let len=self.mapped_1to1.len();
        (len,Some(len))
    }
    fn count(self) -> usize {
        self.mapped_1to1.len()
    }
}


impl std::iter::ExactSizeIterator for TLFieldsIterator{}


///////////////////////////////////////////////////////////////////////////////


#[derive(Copy,Clone,Debug)]
pub struct SplitFieldBoundaries<'a,T>{
    slice:&'a [WithFieldIndex<T>],
    variant_lengths:&'a [u16],
    index:FieldIndex,
    remaining:usize,
}


impl<'a,T> SplitFieldBoundaries<'a,T>{
    pub fn new(slice:&'a [WithFieldIndex<T>],variant_lengths:&'a [u16])->Self{
        Self{
            slice,
            variant_lengths,
            index:FieldIndex{
                variant:0,
                field_pos:0,
            },
            remaining:variant_lengths.iter().map(|&x| x as usize ).sum(),
        }
    }
}
impl<'a,T> Iterator for SplitFieldBoundaries<'a,T>{
    type Item=&'a [WithFieldIndex<T>];

    fn next(&mut self)->Option<Self::Item>{
        if self.remaining==0 {
            return None;
        }

        let ret:&'a [_]=if self.slice.is_empty() {
            &[]
        }else{
            let next_field_ind=self.slice[0].index;

            match self.index.cmp(&next_field_ind) {
                Ordering::Less=>{
                    &[]
                }
                Ordering::Equal=>{
                    let slice=self.slice;
                    let next_field=slice.iter()
                        .position(|x| x.index!=next_field_ind )
                        .unwrap_or(slice.len());
                    let (field_values,rem)=slice.split_at(next_field);
                    self.slice=rem;

                    field_values
                }
                Ordering::Greater=>{
                    panic!("\
                        Expected next_element_index >= current_index.\n\
                        next_element_index:{:?}
                        current_index:{:?}
                    ",
                        next_field_ind,
                        self.index,
                    );
                }
            }
        };

        self.index.increment(self.variant_lengths);
        self.remaining-=1;

        Some(ret)
    }
}


///////////////////////////////////////////////////////////////////////////////


