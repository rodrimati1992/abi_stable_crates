
#[derive(StableAbi)]
#[sabi(inside_abi_stable_crate)]
#[derive(Debug,Copy,Clone,PartialEq,Eq)]
#[repr(transparent)]
pub struct FieldAccessibility{
    bits:u64,
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
    /// on true the field becomes accessible,
    /// on false the field becomes inaccessible.
    #[inline]
    pub const fn set_accessibility(mut self,index:usize,cond:bool)->Self{
        let bits=Self::index_to_bits(index);
        self.bits=[self.bits&!bits,self.bits|bits][cond as usize];
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
}


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
        
        accessibility=accessibility.set_accessibility(3,false);
        assert_eq!(0b_1111_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(5,false);
        assert_eq!(0b_1101_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(6,false);
        assert_eq!(0b_1001_0111,accessibility.bits());
        
        accessibility=accessibility.set_accessibility(10,true);
        assert_eq!(0b_0100_1001_0111,accessibility.bits());
        
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
}