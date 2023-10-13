pub use rfn::*;

mod rfn {
    use crate::StableAbi;

    #[crate::sabi_trait]
    pub trait RFn<'a, In, Out> {
        fn call(&self, input: In) -> Out;
    }
    impl<'a, In: StableAbi, Out: StableAbi, F: Fn(In) -> Out> RFn<'a, In, Out> for F {
        fn call(&self, input: In) -> Out {
            (self)(input)
        }
    }
}

mod rfnmut {
    use crate::StableAbi;

    #[crate::sabi_trait]
    pub trait RFnMut<'a, In, Out> {
        fn call_mut(&mut self, input: In) -> Out;
    }
    impl<'a, In: StableAbi, Out: StableAbi, F: FnMut(In) -> Out> RFnMut<'a, In, Out> for F {
        fn call_mut(&mut self, input: In) -> Out {
            (self)(input)
        }
    }
}

mod rfnonce {
    use crate::StableAbi;

    #[crate::sabi_trait]
    pub trait RFnOnce<'a, In, Out> {
        fn call_once(self, input: In) -> Out;
    }
    impl<'a, In: StableAbi, Out: StableAbi, F: FnOnce(In) -> Out> RFnOnce<'a, In, Out> for F {
        fn call_once(self, input: In) -> Out {
            (self)(input)
        }
    }
}
