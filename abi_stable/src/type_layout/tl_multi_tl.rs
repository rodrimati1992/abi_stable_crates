use crate::{
    abi_stability::stable_abi_trait::TypeLayoutCtor,
    std_types::RSlice,
    type_layout::data_structures::ArrayLen,
};

////////////////////////////////////////////////////////////////////////////////

abi_stable_shared::declare_multi_tl_types!{
    attrs=[
        derive(StableAbi),
        sabi(unsafe_sabi_opaque_fields),
    ]
}


impl TypeLayoutRange{
    pub(crate) fn to_array(&self)->[u16;4]{
        [
            ((self.bits0>>Self::INDEX_0_OFFSET)&Self::INDEX_MASK)as u16,
            ((self.bits0>>Self::INDEX_1_OFFSET)&Self::INDEX_MASK)as u16,
            ((self.bits1>>Self::INDEX_2_OFFSET)&Self::INDEX_MASK)as u16,
            ((self.bits1>>Self::INDEX_3_OFFSET)&Self::INDEX_MASK)as u16,
        ]
    }

    /// Expands this `TypeLayoutRange` into a `MultipleTypeLayouts<'a>`.
    pub fn expand<'a>(&self,type_layouts:&'a [TypeLayoutCtor])->MultipleTypeLayouts<'a>{
        let indices=self.to_array();
        let len=self.len();

        let first_4=ArrayLen{
            array:[
                type_layouts[indices[0] as usize],
                type_layouts[indices[1] as usize],
                type_layouts[indices[2] as usize],
                type_layouts[indices[3] as usize],
            ], 
            len: len.min(4) as u16,
        };

        let remaining=if len <= 4 {
            RSlice::EMPTY
        }else{
            let start_rem=(indices[3]+1)as usize;
            let len_rem=len-4;
            RSlice::from_slice(&type_layouts[start_rem..start_rem+len_rem])
        };
        
        MultipleTypeLayouts{first_4,remaining}
    }
}


////////////////////////////////////////////////////////////////////////////////


/// This stores multiple `TypeLayoutCtor`,some inline and some in a borrowed slice.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub struct MultipleTypeLayouts<'a>{
    first_4:ArrayLen<[TypeLayoutCtor;4]>,
    remaining:RSlice<'a,TypeLayoutCtor>,
}


impl<'a> MultipleTypeLayouts<'a>{
    /// The ammount of TypeLayoutCtor this contains.
    pub fn len(&self)->usize{
        self.first_4.len as usize+self.remaining.len()
    }

    /// Gets an iterator over the TypeLayoutCtor this contains.
    pub fn iter(&self)->MTLIterator<'a> {
        MTLIterator{
            this:*self,
            index:0,
        }
    }
}


#[derive(Clone,Debug)]
pub struct MTLIterator<'a>{
    this:MultipleTypeLayouts<'a>,
    index:usize,
}


impl<'a> Iterator for MTLIterator<'a>{
    type Item=TypeLayoutCtor;

    fn next(&mut self)->Option<TypeLayoutCtor>{
        if self.index < self.this.len() {
            let ret=if self.index < 4 {
                self.this.first_4.array[self.index]
            }else{
                self.this.remaining[self.index-4]
            };
            self.index+=1;
            Some(ret)
        }else{
            None
        }
    }

    fn size_hint(&self)->(usize,Option<usize>){
        let len=self.this.len()-self.index;
        (len,Some(len))
    }
}

impl<'a> ExactSizeIterator for MTLIterator<'a>{}