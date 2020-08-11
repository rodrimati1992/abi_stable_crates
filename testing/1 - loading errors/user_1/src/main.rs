use abi_stable::library::{
    development_utils::compute_library_path,
    LibraryError, RootModule,
};

use testing_interface_1::{
    NonAbiStableLib_Ref, TestingMod_Ref, WithIncompatibleLayout_Ref,
    get_env_vars,
};

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

        if envars.return_error {
            let err = res.err().expect("Expected the library to return an error");
        }else{
            let module = res.unwrap();
            assert_eq!(module.a(), 5);
            assert_eq!(module.b(), 8);
            assert_eq!(module.c(), 13);
        }
    }
}
