/*!
This module runs tests on the C abi as defined by Rust,
to detect whether Rust changed how it deals with zero-sized types.
*/

use crate::std_types::{RBoxError,Tuple2,Tuple3};
use super::LibraryError;

mod functions;

pub use self::functions::{C_ABI_TESTING_FNS,CAbiTestingFns};


/// Tests that the abi (as defined by the compiler) of the functions in 
/// CAbiTestingFns is the same as the caller's.
pub fn run_tests(funcs:&CAbiTestingFns)->Result<(),LibraryError>{
    pair_tests(&funcs)?;
    triple_tests(&funcs)?;
    Ok(())
}

fn make_invalid_cabi_err<T>(expected:T,found:T)->LibraryError
where
    T:std::fmt::Debug
{
    LibraryError::InvalidCAbi{
        expected:RBoxError::from_debug(expected),
        found   :RBoxError::from_debug(found   ),
    }
}

macro_rules! check_roundtrip {
    (
        $funcs:ident,
        $initial_int:expr,
        $composite:expr,

        $ret_fn:ident,
        $take_fn:ident
    ) => {{
        let res=($funcs.$ret_fn)($initial_int);
        let composite=$composite;

        if res!=composite {
            return Err(make_invalid_cabi_err(composite.clone(),res.clone()));
        }
        let int=($funcs.$take_fn)(res);
        if int!=$initial_int {
            return Err(make_invalid_cabi_err(
                Tuple2(composite.clone(),$initial_int),
                Tuple2(res.clone(),int),
            ));
        }
    }}
}

fn pair_tests(funcs:&CAbiTestingFns)->Result<(),LibraryError>{
    check_roundtrip!(funcs,0x1,Tuple2(1,()),ret_pair_a,take_pair_a);
    check_roundtrip!(funcs,0x5,Tuple2(5,()),ret_pair_a,take_pair_a);

    check_roundtrip!(funcs,0x1_0000,Tuple2((),1),ret_pair_b,take_pair_b);
    check_roundtrip!(funcs,0x5_0000,Tuple2((),5),ret_pair_b,take_pair_b);
    Ok(())
}

fn triple_tests(funcs:&CAbiTestingFns)->Result<(),LibraryError>{
    check_roundtrip!(funcs,0x1_0001_0000,Tuple3((),1,1),ret_triple_a,take_triple_a);
    check_roundtrip!(funcs,0x1_0005_0000,Tuple3((),5,1),ret_triple_a,take_triple_a);
    check_roundtrip!(funcs,0x7_0001_0000,Tuple3((),1,7),ret_triple_a,take_triple_a);
    check_roundtrip!(funcs,0x7_0003_0000,Tuple3((),3,7),ret_triple_a,take_triple_a);

    check_roundtrip!(funcs,0x1_0000_0001,Tuple3(1,(),1),ret_triple_b,take_triple_b);
    check_roundtrip!(funcs,0x1_0000_0005,Tuple3(5,(),1),ret_triple_b,take_triple_b);
    check_roundtrip!(funcs,0x7_0000_0001,Tuple3(1,(),7),ret_triple_b,take_triple_b);
    check_roundtrip!(funcs,0x7_0000_0003,Tuple3(3,(),7),ret_triple_b,take_triple_b);

    check_roundtrip!(funcs,0x1_0001,Tuple3(1,1,()),ret_triple_c,take_triple_c);
    check_roundtrip!(funcs,0x1_0005,Tuple3(5,1,()),ret_triple_c,take_triple_c);
    check_roundtrip!(funcs,0x7_0001,Tuple3(1,7,()),ret_triple_c,take_triple_c);
    check_roundtrip!(funcs,0x7_0003,Tuple3(3,7,()),ret_triple_c,take_triple_c);
    Ok(())
}