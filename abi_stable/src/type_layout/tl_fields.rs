use super::*;

use std::{
    slice,
};

/// The layout of a field.
#[repr(C)]
#[derive(Copy, Clone, StableAbi)]
pub struct TLFields {
    /// The field names,separating fields with ";".
    pub names: StaticStr,

    /// The ammount of fields of each variant of the type,
    /// treating structs and unions as a single variant enum.
    pub variant_lengths:StaticSlice<u8>,

    /// Which lifetimes in the struct are referenced in the field type.
    pub lifetime_indices: SliceAndFieldIndices<LifetimeIndex>,

    /// All the function pointer types in the field.
    pub functions:Option<&'static TLFunctions >,

    /// All TLField fields which map 1:1.
    pub field_1to1:StaticSlice<Field1to1>,
}


impl TLFields{
    /// Constructs a `TLFields` from its component parts
    pub const fn new(
        names: &'static str,
        variant_lengths:&'static [u8],
        lifetime_indices: SliceAndFieldIndices<LifetimeIndex>,
        functions:Option<&'static TLFunctions >,
        field_1to1:&'static [Field1to1],
    )->Self{
        Self{
            names:StaticStr::new(names),
            variant_lengths:StaticSlice::new(variant_lengths),
            lifetime_indices,
            functions,
            field_1to1:StaticSlice::new(field_1to1),
        }
    }

    /// Gets an iterator over the fields.
    pub fn iter(&self)->TLFieldsIterator{
        TLFieldsIterator{
            field_names:self.names.as_str().split(FIELD_SPLITTER),
            lifetime_indices:self.lifetime_indices.iter(),
            field_fn_ranges:self.functions
                .map_or(empty_slice(),|x| x.field_fn_ranges.as_slice() )
                .iter(),
            functions:self.functions,
            mapped_1to1:self.field_1to1.as_slice().iter(),
        }
    }
    
    /// Collects the fields into a `Vec<TLField>`.
    pub fn to_vec(&self)->Vec<TLField>{
        self.iter().collect()
    }
}


impl IntoIterator for TLFields {
    type IntoIter=TLFieldsIterator;
    type Item=TLField;

    #[inline]
    fn into_iter(self)->Self::IntoIter{
        self.iter()
    }
}

impl Debug for TLFields{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_struct("TLFields")
         .field("names",&self.names )
         .field("variant_lengths",&self.variant_lengths )
         .field("lifetime_indices",&self.variant_lengths )
         .field("function_count",&self.functions.map_or(0,|x| x.functions.len() ) )
         .field("fields",&self.to_vec())
         .finish()
    }
}


impl Display for TLFields {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        fields_display_formatting(self.iter(),f)
    }
}


impl Eq for TLFields{}
impl PartialEq for TLFields{
    fn eq(&self,other:&Self)->bool{
        self.iter().eq(other.iter())
    }
}


fn fields_display_formatting<I>(iter:I,f:&mut fmt::Formatter<'_>)->fmt::Result
where
    I:IntoIterator<Item=TLField>
{
    for field in iter {
        Display::fmt(&field,f)?;
        writeln!(f)?;
    }
    Ok(())
}


///////////////////////////////////////////////////////////////////////////////


/**
Either a `TLFields` or a static slice of `TLField`.
*/
#[repr(u8)]
#[derive(Copy, Clone,Debug, StableAbi,PartialEq,Eq)]
pub enum TLFieldsOrSlice{
    TLFields(TLFields),
    Slice(StaticSlice<TLField>),
}

impl TLFieldsOrSlice{
    pub const EMPTY:Self=Self::from_slice(&[]);

    /// Constructs a TLFieldsOrSlice from a slice of `TLField` .
    ///
    /// This was defined to make construct TLFieldsOrSlice more easily 
    /// from a `&'static [TLField;N]`.
    pub const fn from_slice(slice:&'static [TLField])->Self{
        TLFieldsOrSlice::Slice(StaticSlice::new(slice))
    }

    /// Gets an iterator over the fields.
    pub fn iter(&self)->TLFOSIter{
        match self {
            TLFieldsOrSlice::TLFields(x)=>TLFOSIter::TLFields(x.iter()),
            TLFieldsOrSlice::Slice(x)=>TLFOSIter::Slice(x.as_slice().iter()),
        }
    }
    
    /// Collects the fields into a `Vec<TLField>`.
    pub fn to_vec(&self)->Vec<TLField>{
        self.iter().collect()
    }

    /// Returns the ammount of fields in this `TLFieldsOrSlice`.
    pub fn len(&self)->usize{
        match self {
            TLFieldsOrSlice::TLFields(x)=>x.field_1to1.len(),
            TLFieldsOrSlice::Slice(x)=>x.len(),
        }
    }

    /// Whether this contains no fields.
    pub fn is_empty(&self)->bool{
        self.len()==0
    }
}


impl IntoIterator for TLFieldsOrSlice {
    type IntoIter=TLFOSIter;
    type Item=TLField;

    #[inline]
    fn into_iter(self)->Self::IntoIter{
        self.iter()
    }
}


impl Display for TLFieldsOrSlice {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        fields_display_formatting(self.iter(),f)
    }
}

impl Default for TLFieldsOrSlice{
    fn default()->Self{
        Self::EMPTY
    }
}


/////////////////////////////////////////////////////



/// A iterator over all the fields of a type definition.
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


/**
A slice of `T`,and a slice of ranges into the first slice,associating each range with a field.
*/
#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct SliceAndFieldIndices<T:'static>{
    pub values: StaticSlice<T>,
    pub field_indices: StaticSlice<WithFieldIndex<u16>>,
}


impl<T> SliceAndFieldIndices<T>{
    /// Constructs the SliceAndFieldIndices from its component parts.
    pub const fn new(
        values: &'static [T],
        field_indices:&'static [WithFieldIndex<u16>],
    )->Self{
        Self{
            values:StaticSlice::new(values),
            field_indices:StaticSlice::new(field_indices),
        }
    }

    /// Iterates over the ranges of T,associated with a field each.
    pub fn iter(&self)->SAFIIter<T>{
        SAFIIter{
            values:self.values.as_slice(),
            field_indices:self.field_indices.as_slice(),
        }
    }
}

impl<T:'static> IntoIterator for SliceAndFieldIndices<T> {
    type IntoIter=SAFIIter<T>;
    type Item=WithFieldIndex<&'static [T]>;

    #[inline]
    fn into_iter(self)->Self::IntoIter{
        self.iter()
    }
}



#[repr(C)]
#[derive(Copy, Clone,Debug)]
pub struct SAFIIter<T:'static>{
    values:&'static [T],
    field_indices:&'static [WithFieldIndex<u16>],
}

impl<T:'static> Iterator for SAFIIter<T>{
    type Item=WithFieldIndex<&'static [T]>;

    fn next(&mut self)->Option<Self::Item>{
        let field_index=self.field_indices.get(0)?;
        let len=self.values.len();
        self.field_indices=&self.field_indices[1..];
        let next_ind=self.field_indices.first().map_or(len,|x| x.value as usize);
        Some(WithFieldIndex{
            index:field_index.index,
            value:&self.values[field_index.value as usize ..next_ind],
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


/**
An index composed of the (variant,field_position) pair.
*/
#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct FieldIndex{
    pub variant:u16,
    pub field_pos:u8,
}

impl FieldIndex {
    pub const fn from_variant_field(variant:u16,field_pos:u8)->Self{
        Self{variant,field_pos}
    }
    pub fn increment(&mut self,variant_lengths:&[u8]){
        let next_field_pos=self.field_pos+1;
        if variant_lengths[self.variant as usize]== next_field_pos {
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

/**
Properties that are associated with all fields.
*/
#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi)]
pub struct Field1to1{
    /// The layout of the field's type.
    ///
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static TypeLayout`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    pub layout: GetTypeLayout,

    /// Whether this field is only a function pointer.
    pub is_function:bool,

    pub field_accessor:FieldAccessor,
}


impl Field1to1{
    pub const fn new(
        layout: GetTypeLayout,
        is_function:bool,
        field_accessor:FieldAccessor,
    )->Self{
        Self{layout,is_function,field_accessor}
    }
}


///////////////////////////////////////////////////////////////////////////////


/**
A pair of (FieldIndex,T).
*/
#[repr(C)]
#[derive(Copy, Clone,Debug, StableAbi,Ord,PartialOrd,Eq,PartialEq)]
pub struct WithFieldIndex<T>{
    pub index:FieldIndex,
    pub value:T,
}

impl<T> WithFieldIndex<T>{
    /// A convenience constructor.
    pub const fn from_vari_field_val(variant:u16,field_pos:u8,value:T)->Self{
        Self{
            index:FieldIndex{variant,field_pos},
            value,
        }
    }    
}


///////////////////////////////////////////////////////////////////////////////


const FIELD_SPLITTER:&'static [char]=&[';','|'];


/**
An iterator over all the fields in a type definition.
*/
#[derive(Clone,Debug)]
pub struct TLFieldsIterator{
    field_names:std::str::Split<'static,&'static [char]>,

    lifetime_indices:SAFIIter<LifetimeIndex>,
    field_fn_ranges:slice::Iter<'static,StartLen>,
    functions:Option<&'static TLFunctions >,

    mapped_1to1:slice::Iter<'static,Field1to1>,
}


impl Iterator for TLFieldsIterator{
    type Item=TLField;

    fn next(&mut self)->Option<TLField>{
        let field_1to1=self.mapped_1to1.next()?.clone();

        Some(TLField{
            name:StaticStr::new(self.field_names.next().unwrap()),
            lifetime_indices:StaticSlice::new(self.lifetime_indices.next().unwrap().value),
            function_range:TLFunctionRange::new(
                self.field_fn_ranges.next().map_or(StartLen::EMPTY,|x|*x),
                self.functions,
            ),
            layout:field_1to1.layout,
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

