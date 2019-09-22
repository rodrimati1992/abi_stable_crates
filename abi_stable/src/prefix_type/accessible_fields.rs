use crate::const_utils::{
    min_usize,
    low_bit_mask_u64,
};

use std::{
    iter::ExactSizeIterator,
    fmt::{self,Debug},
    marker::PhantomData,
};


/// Describes which prefix-type fields are accessible.
///
/// Each field is represented as a bit,where 0 is IsAccessible::No,and 1 s IsAccessible::Yes.
#[must_use="BoolArray is returned by value by every mutating method."]
#[derive(StableAbi)]
#[derive(PartialEq,Eq)]
#[repr(transparent)]
pub struct BoolArray<T>{
    bits:u64,
    _marker:PhantomData<T>,
}

impl<T> Copy for BoolArray<T>{}
impl<T> Clone for BoolArray<T>{
    fn clone(&self)->Self{
        Self{
            bits:self.bits,
            _marker:PhantomData,
        }
    }
}

pub type FieldAccessibility=BoolArray<IsAccessible>;
pub type FieldConditionality=BoolArray<IsConditional>;


impl<T> BoolArray<T>{
    /// Creates a BoolArray where the first `field_count` fields are accessible.
    #[inline]
    pub const fn with_field_count(field_count:usize)->Self{
        Self{
            bits:low_bit_mask_u64(field_count as u32),
            _marker:PhantomData,
        }
    }

    /// Creates a BoolArray from a u64.
    #[inline]
    pub const fn from_u64(bits:u64)->Self{
        Self{
            bits,
            _marker:PhantomData,
        }
    }

    /// Creates a BoolArray where no field is accessible.
    #[inline]
    pub const fn empty()->Self{
        Self{
            bits:0,
            _marker:PhantomData,
        }
    }

    #[inline]
    const fn index_to_bits(index:usize)->u64{
        let index=index as u32;
        [0,1u64.wrapping_shl(index)][(index <= 63) as usize]
    }

    /// Truncates self so that only the first `length` are accessible.
    pub const fn truncated(mut self,length:usize)->Self{
        let mask=Self::with_field_count(length).bits();
        self.bits&=mask;
        self
    }

    #[inline]
    pub const fn bits(self)->u64{
        self.bits
    }

    pub const fn iter_field_count(self,field_count:usize)->BoolArrayIter<T>{
        BoolArrayIter{
            field_count:min_usize(64,field_count),
            bits:self.bits(),
            _marker:PhantomData,
        }
    }

    pub fn is_compatible(self,other:Self,field_count:usize)->bool{
        let all_accessible=Self::with_field_count(field_count);
        let implication=(!self.bits|other.bits)&all_accessible.bits;
        println!(
            "self:{:b}\nother:{:b}\nall_accessible:{:b}\nimplication:{:b}", 
            self.bits,
            other.bits,
            all_accessible.bits,
            implication,
        );
        implication==all_accessible.bits
    }
}


impl FieldAccessibility{
    /// Queries whether the field at the `index` position is accessible.
    #[inline]
    pub const fn is_accessible(self,index:usize)->bool{
        let bits=Self::index_to_bits(index);
        (self.bits&bits)!=0
    }

    /// Sets the accessibility of a field based on `cond`,
    /// on IsAccessible::Yes the field becomes accessible,
    /// on IsAccessible::No the field becomes inaccessible.
    #[inline]
    pub const fn set_accessibility(mut self,index:usize,cond:IsAccessible)->Self{
        let bits=Self::index_to_bits(index);
        self.bits=[self.bits&!bits,self.bits|bits][cond as usize];
        self
    }

}

impl FieldConditionality{
    /// Queries whether the field at the `index` position is accessible.
    #[inline]
    pub const fn is_conditional(self,index:usize)->bool{
        let bits=Self::index_to_bits(index);
        (self.bits&bits)!=0
    }

    /// Sets the accessibility of a field based on `cond`,
    /// on IsConditional::Yes the field becomes conditional,
    /// on IsConditional::No the field becomes unconditional.
    #[inline]
    pub const fn set_conditionality(mut self,index:usize,cond:IsConditional)->Self{
        let bits=Self::index_to_bits(index);
        self.bits=[self.bits&!bits,self.bits|bits][cond as usize];
        self
    }

}


impl<T> Debug for BoolArray<T>
where
    T:BooleanEnum
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_list()
         .entries(self.iter_field_count(64))
         .finish()
    }
}


////////////////////////////////////////////////////////////////////////////////


pub trait BooleanEnum:Debug{
    const NAME:&'static str;

    fn from_bool(b:bool)->Self;
}

////////////////////////////////////////////////////////////////////////////////

/// Whether a field is accessible.
#[derive(StableAbi)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(u8)]
pub enum IsAccessible{
    No=0,
    Yes=1,
}

impl IsAccessible{
    /// Constructs an IsAccessible with a bool saying whether the field is accessible.
    pub const fn new(is_accessible:bool)->Self{
        [IsAccessible::No,IsAccessible::Yes][is_accessible as usize]
    }
    /// Describes whether the field is accessible.
    pub const fn is_accessible(self)->bool{
        self as usize!=0
    }
}

impl BooleanEnum for IsAccessible{
    const NAME:&'static str="IsAccessible";

    fn from_bool(b:bool)->Self{
        Self::new(b)
    }
}


////////////////////////////////////////////////////////////////////////////////

/// Whether a field is conditional,
/// whether it has a `#[sabi(accessible_if=" expression ")]` helper attribute or not.
#[derive(StableAbi)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(u8)]
pub enum IsConditional{
    No=0,
    Yes=1,
}

impl IsConditional{
    /// Constructs an IsConditional with a bool saying whether the field is accessible.
    pub const fn new(is_accessible:bool)->Self{
        [IsConditional::No,IsConditional::Yes][is_accessible as usize]
    }
    /// Describes whether the field is accessible.
    pub const fn is_conditional(self)->bool{
        self as usize!=0
    }
}

impl BooleanEnum for IsConditional{
    const NAME:&'static str="IsConditional";

    fn from_bool(b:bool)->Self{
        Self::new(b)
    }
}


////////////////////////////////////////////////////////////////////////////////



#[derive(Debug,Clone)]
pub struct BoolArrayIter<T>{
    field_count:usize,
    bits:u64,
    _marker:PhantomData<T>,
}


impl<T> BoolArrayIter<T>
where
    T:BooleanEnum,
{
    #[inline]
    fn next_inner<F>(&mut self,f:F)->Option<T>
    where F:FnOnce(&mut Self)->bool
    {
        if self.field_count==0 {
            None
        }else{
            Some(T::from_bool(f(self)))
        }
    }
}
impl<T> Iterator for BoolArrayIter<T>
where
    T:BooleanEnum,
{
    type Item=T;

    fn next(&mut self)->Option<T>{
        self.next_inner(|this|{
            this.field_count-=1;
            let cond=(this.bits&1)!=0;
            this.bits>>=1;
            cond
        })
    }

    #[inline]
    fn size_hint(&self)->(usize,Option<usize>) {
        (self.len(),Some(self.len()))
    }
}


impl<T> DoubleEndedIterator for BoolArrayIter<T>
where
    T:BooleanEnum,
{
    fn next_back(&mut self)->Option<T>{
        self.next_inner(|this|{
            this.field_count-=1;
            (this.bits&(1<<this.field_count))!=0
        })
    }
}

impl<T> ExactSizeIterator for BoolArrayIter<T>
where
    T:BooleanEnum,
{
    #[inline]
    fn len(&self)->usize{
        self.field_count
    }
}



////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////



#[cfg(all(test,not(feature="only_new_tests")))]
mod tests{
    use super::*;
    
    #[test]
    fn with_field_count(){
        for count in 0..=64 {
            let accessibility=BoolArray::with_field_count(count);

            for i in 0..64 {
                assert_eq!(
                    accessibility.is_accessible(i),
                    i < count,
                    "count={} accessibility={:b}",
                    count,
                    accessibility.bits()
                );
            }
        }
    }
    
    #[test]
    fn set_accessibility(){
        let mut accessibility=BoolArray::with_field_count(8);
        assert_eq!(0b_1111_1111,accessibility.bits());
        
        {
            let mut accessibility=accessibility;
            
            accessibility=accessibility.set_accessibility(0,IsAccessible::No);
            assert_eq!(0b_1111_1110,accessibility.bits());

            accessibility=accessibility.set_accessibility(2,IsAccessible::No);
            assert_eq!(0b_1111_1010,accessibility.bits());

            accessibility=accessibility.set_accessibility(1,IsAccessible::No);
            assert_eq!(0b_1111_1000,accessibility.bits());
        }
        
        accessibility=accessibility.set_accessibility(3,IsAccessible::No);
        assert_eq!(0b_1111_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(5,IsAccessible::No);
        assert_eq!(0b_1101_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(6,IsAccessible::No);
        assert_eq!(0b_1001_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(10,IsAccessible::Yes);
        assert_eq!(0b_0100_1001_0111,accessibility.bits());
            
        accessibility=accessibility.set_accessibility(63,IsAccessible::Yes);
        assert_eq!((1<<63)|0b_0100_1001_0111,accessibility.bits());
        
    }
    
    
    #[test]
    fn empty(){
        let accessibility=BoolArray::empty();

        for i in 0..64 {
            assert!(
                !accessibility.is_accessible(i),
                "i={} accessibility={:b}",
                i,
                accessibility.bits()
            );
        }
    }
    
    #[test]
    fn iter_test(){
        let iter=BoolArray::with_field_count(8)
            .set_accessibility(1,IsAccessible::No)
            .set_accessibility(3,IsAccessible::No)
            .iter_field_count(10)
            .map(IsAccessible::is_accessible);
        
        let expected=vec![true,false,true,false,true,true,true,true,false,false];
        let expected_rev=expected.iter().cloned().rev().collect::<Vec<bool>>();

        assert_eq!(
            iter.clone().collect::<Vec<bool>>(),
            expected
        );
        
        assert_eq!(
            iter.clone().rev().collect::<Vec<bool>>(),
            expected_rev
        );
    }
    
}