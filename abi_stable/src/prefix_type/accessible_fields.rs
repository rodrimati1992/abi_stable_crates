use crate::const_utils::{
    min_usize,
};

use std::iter::ExactSizeIterator;


#[must_use="FieldAccessibility is returned by value by every mutating method."]
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(transparent)]
pub struct FieldAccessibility{
    bits:u64,
}

#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(C)]
pub enum IsAccessible{
    No=0,
    Yes=1,
}

impl IsAccessible{
    pub const fn new(is_accessible:bool)->Self{
        [IsAccessible::No,IsAccessible::Yes][is_accessible as usize]
    }
    pub const fn is_accessible(self)->bool{
        self as usize!=0
    }
}


impl FieldAccessibility{
    /// Creates a FieldAccessibility where the first `field_count` fields are accessible.
    #[inline]
    pub const fn with_field_count(field_count:usize)->Self{
        let (n,overflowed)=1u64.overflowing_shl(field_count as u32);
        Self{
            bits:n.wrapping_sub([1,2][overflowed as usize])
        }
    }

    /// Creates a FieldAccessibility where no field is accessible.
    #[inline]
    pub const fn empty()->Self{
        Self{
            bits:0,
        }
    }

    #[inline]
    const fn index_to_bits(index:usize)->u64{
        let index=index as u32;
        [0,1u64.wrapping_shl(index)][(index <= 63) as usize]
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

    /// Truncates self so that only the first `length` are accessible.
    pub const fn truncated(mut self,length:usize)->Self{
        let mask=Self::with_field_count(length).bits();
        self.bits&=mask;
        self
    }

    /// Queries whether the field at the `index` position is accessible.
    #[inline]
    pub const fn is_accessible(self,index:usize)->bool{
        let bits=Self::index_to_bits(index);
        (self.bits&bits)!=0
    }

    #[inline]
    pub const fn bits(self)->u64{
        self.bits
    }

    pub const fn iter_field_count(self,field_count:usize)->FieldAccessibilityIter{
        FieldAccessibilityIter{
            field_count:min_usize(64,field_count),
            bits:self.bits()
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


////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////



#[derive(Debug,Clone)]
pub struct FieldAccessibilityIter{
    pub field_count:usize,
    pub bits:u64,
}


impl FieldAccessibilityIter{
    #[inline]
    fn next_inner<F>(&mut self,f:F)->Option<IsAccessible>
    where F:FnOnce(&mut Self)->bool
    {
        if self.field_count==0 {
            None
        }else{
            Some(IsAccessible::new(f(self)))
        }
    }
}
impl Iterator for FieldAccessibilityIter{
    type Item=IsAccessible;

    fn next(&mut self)->Option<IsAccessible>{
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


impl DoubleEndedIterator for FieldAccessibilityIter{
    fn next_back(&mut self)->Option<IsAccessible>{
        self.next_inner(|this|{
            this.field_count-=1;
            (this.bits&(1<<this.field_count))!=0
        })
    }
}

impl ExactSizeIterator for FieldAccessibilityIter{
    #[inline]
    fn len(&self)->usize{
        self.field_count
    }
}



////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////



#[cfg(test)]
mod tests{
    use super::*;
    
    #[test]
    fn with_field_count(){
        for count in 0..=64 {
            let accessibility=FieldAccessibility::with_field_count(count);

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
        let mut accessibility=FieldAccessibility::with_field_count(8);
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
        let accessibility=FieldAccessibility::empty();

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
        let iter=FieldAccessibility::with_field_count(8)
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