use super::LifetimeIndex;

/// A set of lifetime indices.
pub(crate) struct LifetimeSet{
    set:[u64;4]
}

impl LifetimeSet{
    pub fn new()->Self{
        Self{
            set:[0;4],
        }
    }
    pub fn insert(&mut self,lifetime:LifetimeIndex)->Option<LifetimeIndex>{
        let (i,bit)=Self::get_index_bit( lifetime.bits );
        let bits=&mut self.set[i];
        if (*bits & bit)==0 {
            *bits|=bit;
            None
        }else{
            Some(lifetime)
        }
    }

    #[allow(dead_code)]    
    pub fn remove(&mut self,lifetime:LifetimeIndex)->Option<LifetimeIndex>{
        let (i,bit)=Self::get_index_bit( lifetime.bits );
        let bits=&mut self.set[i];
        if (*bits & bit)==0 {
            None
        }else{
            *bits&=!bit;
            Some(lifetime)
        }
    }
    
    pub fn contains(&self,lifetime:LifetimeIndex)->bool{
        let (i,bit)=Self::get_index_bit( lifetime.bits );
        (self.set[i] & bit)!=0
    }
    
    fn get_index_bit(lt:u8)->(usize,u64){
        ((lt>>6).into(),1<<(lt&63))
    }
}


/////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn set_insert(){
        let mut set=LifetimeSet::new();
        
        let p0=LifetimeIndex::Param(0);
        let p250=LifetimeIndex::Param(250);
        
        assert_eq!(set.insert(LifetimeIndex::STATIC),None);
        assert_eq!(set.insert(LifetimeIndex::STATIC),Some(LifetimeIndex::STATIC));
    
        assert_eq!(set.insert(p0),None);
        assert_eq!(set.insert(p0),Some(p0));
    
        assert_eq!(set.insert(p250),None);
        assert_eq!(set.insert(p250),Some(p250));
    }

    #[test]
    fn set_remove(){
        let mut set=LifetimeSet::new();
        
        let p0=LifetimeIndex::Param(0);
        let p250=LifetimeIndex::Param(250);
        
        assert_eq!(set.remove(p0),None);
        set.insert(p0);
        set.insert(p250);
        assert_eq!(set.remove(p0),Some(p0));
        assert_eq!(set.remove(p250),Some(p250));
        assert_eq!(set.remove(p0),None);
        assert_eq!(set.remove(p250),None);
    }

    #[test]
    fn set_contains(){
        let mut set=LifetimeSet::new();
        
        let p0=LifetimeIndex::Param(0);
        let p250=LifetimeIndex::Param(250);

        assert!(!set.contains(LifetimeIndex::STATIC));
        set.insert(LifetimeIndex::STATIC);
        assert!( set.contains(LifetimeIndex::STATIC));

        assert!(!set.contains(p0));
        set.insert(p0);
        assert!( set.contains(p0));
        
        assert!(!set.contains(p250));
        set.insert(p250);
        assert!( set.contains(p250));
    }
}

