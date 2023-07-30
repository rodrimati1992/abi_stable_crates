use std::fmt::{self, Display};

use readme_interface::{AppenderBox, Appender_TO, BoxedInterface, ExampleLib, ExampleLib_Ref};

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RString, RVec},
    DynTrait,
};

/// The function which exports the root module of the library.
///
/// The root module is exported inside a static of `LibHeader` type,
/// which has this extra metadata:
///
/// - The abi_stable version number used by the dynamic library.
///
/// - A constant describing the layout of the exported root module,and every type it references.
///
/// - A lazily initialized reference to the root module.
///
/// - The constructor function of the root module.
///
#[export_root_module]
pub fn get_library() -> ExampleLib_Ref {
    ExampleLib {
        new_appender,
        new_boxed_interface,
        append_string,
    }
    .leak_into_prefix()
}

/// `DynTrait<_, TheInterface>` is constructed from this type in this example
#[derive(Debug, Clone)]
pub struct StringBuilder {
    pub text: String,
    pub appended: Vec<RString>,
}

impl Display for StringBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl StringBuilder {
    /// Appends the string at the end.
    pub fn append_string(&mut self, string: RString) {
        self.text.push_str(&string);
        self.appended.push(string);
    }
}

#[sabi_extern_fn]
pub fn new_appender() -> AppenderBox<u32> {
    // What `TD_Opaque` does here is specify that the trait object cannot be downcasted,
    // disallowing the `Appender_TO` from being unwrapped back into an `RVec<u32>`
    // when the `trait_object.obj.*_downcast_*()` methods are used.
    //
    // To be able to unwrap a `#[sabi_trait]` trait object back into the type it
    // was constructed with, you must:
    //
    // - Have a type that implements `std::anu::Any`
    // (it requires that the type doesn't borrow anything).
    //
    // - Pass `TD_CanDowncast` instead of `TD_Opaque` to
    // `Appender_TO::{from_const, from_value,from_ptr}`.
    //
    // - Unerase the trait object back into the original type with
    //     `trait_object.obj.downcast_into::<RVec<u32>>().unwrap()`
    //     (or the other downcasting methods).
    //
    // Downcasting a trait object will fail in any of these conditions:
    //
    // - It wasn't constructed in the same dynamic library.
    //
    // - It's not the same type.
    //
    // - It was constructed with `TD_Opaque`.
    //
    Appender_TO::from_value(RVec::new(), TD_Opaque)
}

/// Constructs a BoxedInterface.
#[sabi_extern_fn]
fn new_boxed_interface() -> BoxedInterface<'static> {
    DynTrait::from_value(StringBuilder {
        text: "".into(),
        appended: vec![],
    })
}

/// Appends a string to the erased `StringBuilder`.
#[sabi_extern_fn]
fn append_string(wrapped: &mut BoxedInterface<'_>, string: RString) {
    wrapped
        .downcast_as_mut::<StringBuilder>() // Returns `Result<&mut StringBuilder, _>`
        .unwrap() // Returns `&mut StringBuilder`
        .append_string(string);
}
