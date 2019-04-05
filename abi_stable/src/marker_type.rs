use std::{marker::PhantomData, rc::Rc};

pub struct SyncSend;

pub struct UnsyncUnsend {
    _marker: PhantomData<Rc<()>>,
}
