use std::{
    any::Any,
    panic::{catch_unwind, AssertUnwindSafe, Location},
};

pub type ThreadError = Box<dyn Any + Send + 'static>;

#[derive(Debug, Clone)]
pub struct ShouldHavePanickedAt {
    pub span: &'static std::panic::Location<'static>,
}

#[track_caller]
pub fn must_panic<F, R>(f: F) -> Result<ThreadError, ShouldHavePanickedAt>
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(_) => Err(ShouldHavePanickedAt {
            span: Location::caller(),
        }),
        Err(e) => Ok(e),
    }
}

#[test]
fn test_must_panic() {
    assert!(must_panic(|| panic!()).is_ok());
    assert!(must_panic(|| ()).is_err());
}
