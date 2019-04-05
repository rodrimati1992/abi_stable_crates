use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe},
};

#[derive(Debug, Clone)]
pub struct FileSpan {
    pub file: &'static str,
    pub line: &'static str,
}

pub type ThreadError = Box<dyn Any + Send + 'static>;

#[derive(Debug, Clone)]
pub struct ShouldHavePanickedAt {
    pub span: FileSpan,
}

macro_rules! file_span {
    () => {{
        use crate::test_utils::FileSpan;
        FileSpan {
            file: file!(),
            line: file!(),
        }
    }};
}

pub fn must_panic<F, R>(span: FileSpan, f: F) -> Result<ThreadError, ShouldHavePanickedAt>
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(_) => Err(ShouldHavePanickedAt { span }),
        Err(e) => Ok(e),
    }
}

#[test]
fn test_must_panic() {
    assert!(must_panic(file_span!(), || panic!()).is_ok());
    assert!(must_panic(file_span!(), || ()).is_err());
}
