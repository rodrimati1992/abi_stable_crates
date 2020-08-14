use abi_stable::library::{
    development_utils::compute_library_path,
    LibraryError, RootModule, RootModuleError,
};

use testing_interface_1::{
    NonAbiStableLib_Ref, ReturnWhat, TestingMod_Ref, WithIncompatibleLayout_Ref,
    get_env_vars,
};

use std::fmt;

fn main() {
    let target: &std::path::Path = "../../../target/".as_ref();

    let envars = get_env_vars();

    println!("app: {:?}", envars);
    
    {
        let err = WithIncompatibleLayout_Ref::load_from_directory("foo/bar/bar".as_ref())
            .err()
            .unwrap();
        assert!(
            core_extensions::matches!( LibraryError::OpenError{..} = err ),
            "{:?}",
            err,
        );
    }

    {
        let library_path=
            compute_library_path::<WithIncompatibleLayout_Ref>(target).unwrap();
        let err = NonAbiStableLib_Ref::load_from_directory(&library_path).err().unwrap();
        assert!(
            core_extensions::matches!( LibraryError::GetSymbolError{..} = err ),
            "{:?}",
            err,
        );
    }

    {
        let library_path=
            compute_library_path::<WithIncompatibleLayout_Ref>(target).unwrap();

        let err = WithIncompatibleLayout_Ref::load_from_directory(&library_path)
            .err()
            .unwrap();

        assert!(
            core_extensions::matches!( LibraryError::AbiInstability(_) = err ),
            "{:#}",
            err,
        );

        // Doing this to make sure that the error formatting is not optimized out.
        let formatted = format!("{0} {0:?}", err);
        println!(
            "sum of bytes in the error: {}",
            formatted.bytes().map(|x| x as u64 ).sum::<u64>()
        );
        
    }

    {    
        let library_path=compute_library_path::<TestingMod_Ref>(target).unwrap();
        let res=TestingMod_Ref::load_from_directory(&library_path);

        match envars.return_what {
            ReturnWhat::Ok=>{
                let module = res.unwrap();
                assert_eq!(module.a(), 5);
                assert_eq!(module.b(), 8);
                assert_eq!(module.c(), 13);
            }
            ReturnWhat::Error|ReturnWhat::Panic=>{
                let err = res.err().expect("Expected the library to return an error");

                if let LibraryError::RootModule{err: rm_err, ..} = &err {

                    assert!(
                        core_extensions::matches!(
                             (ReturnWhat::Error, RootModuleError::Returned{..})
                            |(ReturnWhat::Panic, RootModuleError::Unwound{..})
                            = (&envars.return_what, rm_err)
                        ),
                        "return_what: {:?}\nerror: {:?}",
                        envars.return_what,
                        rm_err,
                    );

                    print_error_sum(line!(), rm_err);
                } else {
                    panic!(
                        "Expected a LibraryError::RootModule, found this instead:\n{:#?}",
                        err,
                    );
                }
                print_error_sum(line!(), err);
            }
        }
        println!(
            "\n{S}{S}\n\nFinished successfully\n\n{S}{S}\n",
            S="----------------------------------------",
        );
    }
}

fn print_error_sum<E: fmt::Debug + fmt::Display>(line: u32, e: E) {
    let formatted = format!("{0} {0:?}", e);
    let sum = formatted.bytes()
        .map(|x| x as u64 )
        .sum::<u64>();
    println!("{}: sum of bytes in the error: {}", line, sum);
}
