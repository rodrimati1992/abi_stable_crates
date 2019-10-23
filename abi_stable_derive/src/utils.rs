use std::{
    mem::{self,ManuallyDrop},
    ops::{Deref,DerefMut},
    ptr,
    time::Instant,
};

use abi_stable_shared::test_utils::{FileSpan};

use core_extensions::measure_time::MyDuration;

pub(crate) use as_derive_utils::utils::{
    join_spans,
    dummy_ident,
    type_from_ident,
    expr_from_ident,
    expr_from_int,
    take_manuallydrop,
    uint_lit,
    LinearResult,
    SynPathExt,
    SynResultExt,
};


#[allow(dead_code)]
pub struct PrintDurationOnDrop{
    start:Instant,
    file_span:FileSpan,
}

impl PrintDurationOnDrop{
    #[allow(dead_code)]
    pub fn new(file_span:FileSpan)->Self{
        Self{
            start:Instant::now(),
            file_span,
        }
    }
}

impl Drop for PrintDurationOnDrop{
    fn drop(&mut self){
        let span=self.file_span;
        let dur:MyDuration=self.start.elapsed().into();
        println!("{}-{}:taken {} to run",span.file,span.line,dur);
    }
}


