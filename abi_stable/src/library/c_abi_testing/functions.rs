use super::*;

/////////////////////////////////////

#[repr(C)]
#[derive(StableAbi)]
pub struct CAbiTestingFns{
    pub(crate) take_pair_a:extern "C" fn(Tuple2<u16,()>)->u32,
    pub(crate) take_pair_b:extern "C" fn(Tuple2<(),u16>)->u32,
    pub(crate) ret_pair_a:extern "C" fn(u32)->Tuple2<u16,()>,
    pub(crate) ret_pair_b:extern "C" fn(u32)->Tuple2<(),u16>,
    
    pub(crate) take_triple_a:extern "C" fn(Tuple3<(),u16,u16>)->u64,
    pub(crate) take_triple_b:extern "C" fn(Tuple3<u16,(),u16>)->u64,
    pub(crate) take_triple_c:extern "C" fn(Tuple3<u16,u16,()>)->u64,
    pub(crate) ret_triple_a:extern "C" fn(u64)->Tuple3<(),u16,u16>,
    pub(crate) ret_triple_b:extern "C" fn(u64)->Tuple3<u16,(),u16>,
    pub(crate) ret_triple_c:extern "C" fn(u64)->Tuple3<u16,u16,()>,
}

/////////////////////////////////////

/// Functions used to test that the C abi is the same in both the library and the loader.
pub const C_ABI_TESTING_FNS:&'static CAbiTestingFns=&CAbiTestingFns{
    take_pair_a,
    take_pair_b,
    ret_pair_a,
    ret_pair_b,
    take_triple_a,
    take_triple_b,
    take_triple_c,
    ret_triple_a,
    ret_triple_b,
    ret_triple_c,
};


pub(crate) extern "C" fn take_pair_a(pair:Tuple2<u16,()>)->u32{
    (pair.0 as u32)
}
pub(crate) extern "C" fn take_pair_b(pair:Tuple2<(),u16>)->u32{
    (pair.1 as u32)<<16
}
pub(crate) extern "C" fn ret_pair_a(n:u32)->Tuple2<u16,()>{
    Tuple2(n as u16,())
}
pub(crate) extern "C" fn ret_pair_b(n:u32)->Tuple2<(),u16>{
    Tuple2((),(n>>16)as u16)
}


pub(crate) extern "C" fn take_triple_a(triple:Tuple3<(),u16,u16>)->u64{
    ((triple.1 as u64)<<16)+
    ((triple.2 as u64)<<32)
}
pub(crate) extern "C" fn take_triple_b(triple:Tuple3<u16,(),u16>)->u64{
    ((triple.0 as u64))+
    ((triple.2 as u64)<<32)
}
pub(crate) extern "C" fn take_triple_c(triple:Tuple3<u16,u16,()>)->u64{
    ((triple.0 as u64))+
    ((triple.1 as u64)<<16)
}
pub(crate) extern "C" fn ret_triple_a(n:u64)->Tuple3<(),u16,u16>{
    Tuple3(
        (),
        (n>>16) as u16,
        (n>>32) as u16,
    )
}
pub(crate) extern "C" fn ret_triple_b(n:u64)->Tuple3<u16,(),u16>{
    Tuple3(
        n as u16,
        (),
        (n>>32) as u16,
    )
}
pub(crate) extern "C" fn ret_triple_c(n:u64)->Tuple3<u16,u16,()>{
    Tuple3(
        n as u16,
        (n>>16) as u16,
        (),
    )
}
