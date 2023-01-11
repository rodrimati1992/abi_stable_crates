use abi_stable::StableAbi;

#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 2,
)))]
#[sabi(with_constructor)]
pub enum OkSize {
    Foo,
    Bar,
    Baz(u8),
}

// The size of the storage is actually `align_of::<usize>()`,
// because the alignment defaults to that of a usize
#[repr(u8)]
#[derive(StableAbi)]
#[sabi(kind(WithNonExhaustive(
    size = 1,
    assert_nonexhaustive = OkSizeBecauseAlignIsUsize,
)))]
#[sabi(with_constructor)]
pub enum OkSizeBecauseAlignIsUsize {
    Foo,
    Bar,
    Baz(u8),
}

fn main(){}