use abi_stable::std_types::RVec;

use readme_interface::{
    load_root_module_in_directory, AppenderBox, Appender_TO, BoxedInterface, ExampleLib_Ref,
};

fn main() {
    // The type annotation is for the reader
    let library: ExampleLib_Ref = load_root_module_in_directory("../../../target/debug".as_ref())
        .unwrap_or_else(|e| panic!("{}", e));

    {
        /////////////////////////////////////////////////////////////////////////////////
        //
        //       This block demonstrates `#[sabi_trait]` generated trait objects
        //
        ////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut appender: AppenderBox<u32> = library.new_appender()();
        appender.push(100);
        appender.push(200);

        // The primary way to use the methods in the trait is through the inherent methods on
        // the ffi-safe trait object.
        Appender_TO::push(&mut appender, 300);
        appender.append(vec![500, 600].into());
        assert_eq!(
            appender.into_rvec(),
            RVec::from(vec![100, 200, 300, 500, 600])
        );
    }
    {
        ///////////////////////////////////////////////////////////////////////////////////
        //
        //  This block demonstrates the `DynTrait<>` trait object.
        //
        //  `DynTrait` is used here as a safe opaque type which can only be unwrapped back to
        //  the original type in the dynamic library that constructed the `DynTrait` itself.
        //
        ////////////////////////////////////////////////////////////////////////////////////

        // The type annotation is for the reader
        let mut unwrapped: BoxedInterface = library.new_boxed_interface()();

        library.append_string()(&mut unwrapped, "Hello".into());
        library.append_string()(&mut unwrapped, ", world!".into());

        assert_eq!(&*unwrapped.to_string(), "Hello, world!");
    }

    println!("success");
}
