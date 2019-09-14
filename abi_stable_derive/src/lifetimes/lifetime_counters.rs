use super::LifetimeIndex;


use std::fmt::{self,Debug};


/// A set of lifetime indices.
pub(crate) struct LifetimeCounters{
    set:[u8;64],
    max_index:u8,
}

const MASK:u8=0b11;
const MAX_VAL:u8=3;

impl LifetimeCounters{
    pub fn new()->Self{
        Self{
            set:[0;64],
            max_index:0,
        }
    }
    /// Increments the counter for the `lifetime` lifetime,stopping at 3.
    pub fn increment(&mut self,lifetime:LifetimeIndex)->u8{
        let (i,shift)=Self::get_index_shift( lifetime.bits );
        self.max_index=(i as u8).max(self.max_index);
        let bits=&mut self.set[i];
        let mask=MASK << shift;
        if (*bits&mask)==mask {
            MAX_VAL
        }else{
            *bits+=1<<shift;
            (*bits>>shift)&MASK
        }
    }

    pub fn get(&self,lifetime:LifetimeIndex)->u8{
        let (i,shift)=Self::get_index_shift( lifetime.bits );
        (self.set[i] >> shift) & MASK
    }
    
    fn get_index_shift(lt:u8)->(usize,u8){
        (
            (lt>>2).into(),
            (lt&3)<<1
        )
    }
}



impl Debug for LifetimeCounters{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_list()
         .entries(self.set[..usize::from(self.max_index)].iter().cloned().map(U8Wrapper))
         .finish()
    }
}


#[repr(transparent)]
struct U8Wrapper(u8);

impl fmt::Debug for U8Wrapper{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        fmt::Binary::fmt(&self.0,f)
    }
}



#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_counting(){
        let mut counters=LifetimeCounters::new();

        let lts=vec![
            LifetimeIndex::Param(0),
            LifetimeIndex::Param(1),
            LifetimeIndex::Param(2),
            LifetimeIndex::Param(3),
            LifetimeIndex::Param(4),
            LifetimeIndex::Param(5),
            LifetimeIndex::Param(6),
            LifetimeIndex::Param(7),
            LifetimeIndex::Param(8),
            LifetimeIndex::Param(9),
            LifetimeIndex::ANONYMOUS,
            LifetimeIndex::STATIC,
            LifetimeIndex::NONE,
        ];

        for lt in lts {
            for i in 1..=3 {
                assert_eq!(counters.get(lt), i-1);
                assert_eq!(counters.increment(lt), i);
                assert_eq!(counters.get(lt), i);
            }
        }
    }
}