//! This module runs tests on the C abi as defined by Rust,
//! to detect whether Rust changed how it deals with zero-sized types.

use super::LibraryError;
use crate::std_types::{RBoxError, Tuple2, Tuple3};

// Types used in tests from this module
mod types;

mod functions;

#[macro_use]
mod c_abi_testing_macros;

use self::types::MyUnit;

pub use self::functions::{CAbiTestingFns, C_ABI_TESTING_FNS};

/// Tests that the abi (as defined by the compiler) of the functions in
/// CAbiTestingFns is the same as the caller's.
pub fn run_tests(funcs: &CAbiTestingFns) -> Result<(), LibraryError> {
    pair_tests(funcs)?;
    triple_tests(funcs)?;
    two_pair_tests(funcs)?;
    mixed_units_test(funcs)?;
    Ok(())
}

fn make_invalid_cabi_err<T>(expected: T, found: T) -> LibraryError
where
    T: std::fmt::Debug,
{
    LibraryError::InvalidCAbi {
        expected: RBoxError::from_debug(&expected),
        found: RBoxError::from_debug(&found),
    }
}

fn pair_tests(funcs: &CAbiTestingFns) -> Result<(), LibraryError> {
    check_roundtrip!(funcs, 0x1, (a = Tuple2(1, ())), ret_pair_a, take_pair_a);
    check_roundtrip!(funcs, 0x5, (a = Tuple2(5, ())), ret_pair_a, take_pair_a);

    check_roundtrip!(
        funcs,
        0x1_0000,
        (a = Tuple2((), 1)),
        ret_pair_b,
        take_pair_b
    );
    check_roundtrip!(
        funcs,
        0x5_0000,
        (a = Tuple2((), 5)),
        ret_pair_b,
        take_pair_b
    );
    Ok(())
}

fn triple_tests(funcs: &CAbiTestingFns) -> Result<(), LibraryError> {
    check_roundtrip!(
        funcs,
        0x1_0001_0000,
        (a = Tuple3((), 1, 1)),
        ret_triple_a,
        take_triple_a
    );
    check_roundtrip!(
        funcs,
        0x1_0005_0000,
        (a = Tuple3((), 5, 1)),
        ret_triple_a,
        take_triple_a
    );
    check_roundtrip!(
        funcs,
        0x7_0001_0000,
        (a = Tuple3((), 1, 7)),
        ret_triple_a,
        take_triple_a
    );
    check_roundtrip!(
        funcs,
        0x7_0003_0000,
        (a = Tuple3((), 3, 7)),
        ret_triple_a,
        take_triple_a
    );

    check_roundtrip!(
        funcs,
        0x1_0000_0001,
        (a = Tuple3(1, (), 1)),
        ret_triple_b,
        take_triple_b
    );
    check_roundtrip!(
        funcs,
        0x1_0000_0005,
        (a = Tuple3(5, (), 1)),
        ret_triple_b,
        take_triple_b
    );
    check_roundtrip!(
        funcs,
        0x7_0000_0001,
        (a = Tuple3(1, (), 7)),
        ret_triple_b,
        take_triple_b
    );
    check_roundtrip!(
        funcs,
        0x7_0000_0003,
        (a = Tuple3(3, (), 7)),
        ret_triple_b,
        take_triple_b
    );

    check_roundtrip!(
        funcs,
        0x1_0001,
        (a = Tuple3(1, 1, ())),
        ret_triple_c,
        take_triple_c
    );
    check_roundtrip!(
        funcs,
        0x1_0005,
        (a = Tuple3(5, 1, ())),
        ret_triple_c,
        take_triple_c
    );
    check_roundtrip!(
        funcs,
        0x7_0001,
        (a = Tuple3(1, 7, ())),
        ret_triple_c,
        take_triple_c
    );
    check_roundtrip!(
        funcs,
        0x7_0003,
        (a = Tuple3(3, 7, ())),
        ret_triple_c,
        take_triple_c
    );
    Ok(())
}

fn two_pair_tests(funcs: &CAbiTestingFns) -> Result<(), LibraryError> {
    let funcs = anon_struct! {
        ret_2_pairs_a:|n|(funcs.ret_2_pairs_a)(n).into_tuple(),
        ret_2_pairs_b:|n|(funcs.ret_2_pairs_b)(n).into_tuple(),
        take_2_pairs_a:funcs.take_2_pairs_a,
        take_2_pairs_b:funcs.take_2_pairs_b,
    };

    check_roundtrip!(
        funcs,
        0x1_0000_0005_0000,
        (a = Tuple2((), 5), b = Tuple2((), 1)),
        ret_2_pairs_a,
        take_2_pairs_a,
    );
    check_roundtrip!(
        funcs,
        0xF_0000_000A_0000,
        (a = Tuple2((), 10), b = Tuple2((), 15)),
        ret_2_pairs_a,
        take_2_pairs_a,
    );

    check_roundtrip!(
        funcs,
        0x0_0007_0000_0003,
        (a = Tuple2(3, ()), b = Tuple2(7, ())),
        ret_2_pairs_b,
        take_2_pairs_b,
    );
    check_roundtrip!(
        funcs,
        0x0_0002_0000_000B,
        (a = Tuple2(11, ()), b = Tuple2(2, ())),
        ret_2_pairs_b,
        take_2_pairs_b,
    );
    Ok(())
}

fn mixed_units_test(funcs: &CAbiTestingFns) -> Result<(), LibraryError> {
    let single_test = |n: u64| {
        let a = n as u16;
        let b = (n >> 16) as u16;
        let c = (n >> 32) as u16;
        let d = (n >> 48) as u16;

        let res = (funcs.mixed_units)(a, MyUnit, b, MyUnit, c, MyUnit, d);
        if res != n {
            Err(make_invalid_cabi_err(n, res))
        } else {
            Ok(())
        }
    };

    single_test(0x0)?;
    single_test(0x1)?;
    single_test(0x2_0003)?;
    single_test(0x4_0005_0006)?;
    single_test(0x7_0008_0009_000A)?;
    Ok(())
}
