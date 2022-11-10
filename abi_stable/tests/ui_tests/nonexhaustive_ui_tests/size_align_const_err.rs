use abi_stable::{
    nonexhaustive_enum::NonExhaustiveFor as NEFor,
    StableAbi,
};

#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 1,
    align = 1,
)))]
#[sabi(with_constructor)]
pub enum TooLarge<T = u8> {
    Foo,
    Bar,
    Baz(T),
}

const _: () = { std::mem::forget(NEFor::new(<TooLarge>::Foo)); };


#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 32,
    align = 1,
)))]
#[sabi(with_constructor)]
pub enum Unaligned<T = u64> {
    Foo,
    Bar,
    Baz(T),
}

const _: () = { std::mem::forget(NEFor::new(<Unaligned>::Foo)); };

#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = {one()},
    align = {one()},
)))]
#[sabi(with_constructor)]
pub enum UnalignedAndTooLarge<T = u64> {
    Foo,
    Bar,
    Baz(T),
}

const _: () = { std::mem::forget(NEFor::new(<UnalignedAndTooLarge>::Foo)); };

const fn one() -> usize {
    1
}


fn main(){}