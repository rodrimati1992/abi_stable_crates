#![allow(clippy::derive_partial_eq_without_eq)]

use crate::{
    prefix_type::{PrefixRef, WithMetadata},
    StableAbi,
};

mod cond_fields {
    use super::*;

    /// This type is used in prefix type examples.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix(prefix_ref = Module_Ref, prefix_fields = Module_Prefix)))]
    pub struct Module {
        #[sabi(accessible_if = true)]
        pub first: usize,

        #[sabi(last_prefix_field)]
        pub second: usize,

        #[sabi(accessible_if = true)]
        pub third: usize,

        pub fourth: usize,
    }

    impl std::ops::Deref for Module_Prefix {
        type Target = DerefTo;

        fn deref(&self) -> &DerefTo {
            &DerefTo {
                first: "5",
                second: "8",
                third: "13",
                fourth: "21",
            }
        }
    }

    pub const MOD_VAL: &WithMetadata<Module> = &WithMetadata::new(Module {
        first: 5,
        second: 8,
        third: 13,
        fourth: 21,
    });

    pub const PREFIX: PrefixRef<Module_Prefix> = MOD_VAL.static_as_prefix();
}

pub struct DerefTo {
    pub first: &'static str,
    pub second: &'static str,
    pub third: &'static str,
    pub fourth: &'static str,
}

#[test]
fn prefix_field_vis() {
    use cond_fields::{Module_Ref, PREFIX};

    let pref = PREFIX.prefix();
    let modref = Module_Ref(PREFIX);

    // Exploiting the fact that the compiler does a deref coercion
    // if it can't find a pub field with a given name.

    assert_eq!(pref.first, "5");
    assert_eq!(modref.first(), Some(5));

    assert_eq!(pref.second, 8);
    assert_eq!(modref.second(), 8);

    assert_eq!(pref.third, "13");
    assert_eq!(modref.third(), Some(13));

    assert_eq!(pref.fourth, "21");
    assert_eq!(modref.fourth(), Some(21));
}

////////////////////////////////////////////////////////////////////////////////

mod different_alignments {
    use super::*;

    /// This type is used in prefix type examples.
    #[repr(C)]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix(prefix_ref = Module_Ref, prefix_fields = Module_Prefix)))]
    pub struct Module {
        pub f0: u64,
        #[sabi(last_prefix_field)]
        pub f1: u8,
        pub f2: u16,
        pub f3: u32,
        pub f4: u64,
        pub f5: u32,
        pub f6: u16,
        pub f7: u8,
    }

    pub const MOD_VAL: &WithMetadata<Module> = &WithMetadata::new(Module {
        f0: 5,
        f1: 8,
        f2: 13,
        f3: 21,
        f4: 34,
        f5: 55,
        f6: 89,
        f7: 144,
    });

    pub const PREFIX: PrefixRef<Module_Prefix> = MOD_VAL.static_as_prefix();
}

/// Making sure that suffix fields with different alignments are accessed correctly.
#[test]
fn access_different_alignments() {
    use different_alignments::{Module_Ref, PREFIX};

    let modref = Module_Ref(PREFIX);

    assert_eq!(modref.f0(), 5);
    assert_eq!(modref.f1(), 8);
    assert_eq!(modref.f2(), Some(13));
    assert_eq!(modref.f3(), Some(21));
    assert_eq!(modref.f4(), Some(34));
    assert_eq!(modref.f5(), Some(55));
    assert_eq!(modref.f6(), Some(89));
    assert_eq!(modref.f7(), Some(144));
}

////////////////////////////////////////////////////////////////////////////////

#[repr(C, align(32))]
#[derive(StableAbi, Debug, Copy, Clone, PartialEq)]
pub struct AlignTo32<T>(pub T);

#[repr(C, align(64))]
#[derive(StableAbi, Debug, Copy, Clone, PartialEq)]
pub struct AlignTo64<T>(pub T);

mod overaligned {
    use super::*;

    /// This type is used in prefix type examples.
    #[repr(C, align(32))]
    #[derive(StableAbi)]
    #[sabi(kind(Prefix))]
    pub struct Module {
        pub f0: u64,
        pub f1: u8,
        #[sabi(last_prefix_field)]
        pub f2: u16,
        pub f3: AlignTo32<u32>,
        pub f4: AlignTo64<u32>,
        pub f5: u16,
        pub f6: u8,
    }

    pub const MOD_VAL: &WithMetadata<Module> = &WithMetadata::new(Module {
        f0: 5,
        f1: 8,
        f2: 13,
        f3: AlignTo32(21),
        f4: AlignTo64(34),
        f5: 55,
        f6: 89,
    });

    pub const PREFIX: PrefixRef<Module_Prefix> = MOD_VAL.static_as_prefix();
}

/// Making sure that suffix fields with different alignments are accessed correctly.
#[test]
fn access_overaligned_fields() {
    use overaligned::{Module_Ref, PREFIX};

    let modref = Module_Ref(PREFIX);

    assert_eq!(modref.f0(), 5);
    assert_eq!(modref.f1(), 8);
    assert_eq!(modref.f2(), 13);
    assert_eq!(modref.f3(), Some(AlignTo32(21)));
    assert_eq!(modref.f4(), Some(AlignTo64(34)));
    assert_eq!(modref.f5(), Some(55));
    assert_eq!(modref.f6(), Some(89));
}
