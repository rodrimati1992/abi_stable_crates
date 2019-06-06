use std::time::Instant;

use core_extensions::measure_time::MyDuration;
use quote::{ToTokens};
use proc_macro2::TokenStream as TokenStream2;

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub struct NoTokens;

impl ToTokens for NoTokens {
    fn to_tokens(&self, _: &mut TokenStream2) {}
}




#[derive(Debug, Clone,Copy)]
pub struct FileSpan {
    pub file: &'static str,
    pub line: u32,
}
macro_rules! file_span {
    () => {{
        use crate::utils::FileSpan;
        FileSpan {
            file: file!(),
            line: line!(),
        }
    }};
}



pub struct PrintDurationOnDrop{
    start:Instant,
    file_span:FileSpan,
}

impl PrintDurationOnDrop{
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