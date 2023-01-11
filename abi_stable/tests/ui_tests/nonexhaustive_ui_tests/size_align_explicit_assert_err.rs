#![allow(warnings, unused_unsafe)]

use abi_stable::StableAbi;

#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 1,
    align = 1,
    assert_nonexhaustive = TooLarge,
)))]
#[sabi(with_constructor)]
pub enum TooLarge {
    Foo,
    Bar,
    Baz(u8),
}

#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 32,
    align = 1,
    assert_nonexhaustive = Unaligned,
)))]
#[sabi(with_constructor)]
pub enum Unaligned {
    Foo,
    Bar,
    Baz(u64),
}


#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 1,
    align = 1,
    assert_nonexhaustive = UnalignedAndTooLarge,
)))]
#[sabi(with_constructor)]
pub enum UnalignedAndTooLarge {
    Foo,
    Bar,
    Baz(u64),
}

fn main(){}