use std::time::Instant;

use abi_stable_shared::test_utils::FileSpan;

pub(crate) use as_derive_utils::utils::{
    dummy_ident,
    expr_from_ident,
    expr_from_int,
    join_spans,
    type_from_ident,
    //take_manuallydrop,
    uint_lit,
    LinearResult,
    SynPathExt,
    SynResultExt,
};

#[allow(dead_code)]
pub struct PrintDurationOnDrop {
    start: Instant,
    file_span: FileSpan,
}

impl PrintDurationOnDrop {
    #[allow(dead_code)]
    pub fn new(file_span: FileSpan) -> Self {
        Self {
            start: Instant::now(),
            file_span,
        }
    }
}

impl Drop for PrintDurationOnDrop {
    fn drop(&mut self) {
        let span = self.file_span;
        let dur = self.start.elapsed();
        println!("{}-{}:taken {:?} to run", span.file, span.line, dur);
    }
}
