use std::{
    cmp::{PartialEq,Eq},
};


////////////////////////////////////////////////////////////////////////////////

#[repr(C)]
#[derive(Debug, Copy, Clone, StableAbi)]
pub struct ArrayLen<A>{
    pub len:u16,
    pub array:A,
}

impl<A> ArrayLen<A> {
    pub fn len(&self)->usize{
        self.len as usize
    }
}


impl<A,T> PartialEq for ArrayLen<A>
where
    A:ArrayTrait<Elem=T>,
    T:PartialEq,
{
    fn eq(&self,other:&Self)->bool{
        let t_slice=&self.array.as_slice()[..self.len as usize];
        let o_slice=&other.array.as_slice()[..other.len as usize];
        t_slice==o_slice
    }
}


impl<A,T> Eq for ArrayLen<A>
where
    A:ArrayTrait<Elem=T>,
    T:Eq,
{}

////////////////////////////////////////////////////////////////////////////////


pub trait ArrayTrait{
    type Elem;

    fn as_slice(&self)->&[Self::Elem];
}


macro_rules! impl_stable_abi_array {
    ($($size:expr),*)=>{
        $(
            impl<T> ArrayTrait for [T;$size] {
                type Elem=T;

                fn as_slice(&self)->&[T]{
                    self
                }
            }
        )*
    }
}
        
impl_stable_abi_array! {
    00,01,02,03,04,05,06,07,08,09,
    10,11,12,13,14,15,16,17,18,19,
    20,21,22,23,24,25,26,27,28,29,
    30,31,32
}