/*!
Types,traits,and functions used by prefix-types.

*/


/// A trait implemented by all prefix-types,providing some metadata about them.
pub trait PrefixTypeTrait{
    fn layout()->&'static TypeLayout;
    fn metadata()->PrefixTypeMetadata{
        PrefixTypeMetadata::new(Self::layout())
    }
}


pub struct PrefixTypeMetadata{
    /// This is the ammount of fields on the prefix of the struct,
    /// which is always the same for the same type,regardless of which library it comes from.
    pub prefix_field_count:usize,

    pub fields:StaticSlice<TLField>,

    /// The layout of the struct,for error messages.
    pub layout:&'static TypeLayout,
}


impl PrefixTypeMetadata{
    pub fn new(layout:&'static TypeLayout)->Self{
        let (first_suffix_field,fields)=match layout.data {
            PrefixType{first_suffix_field,fields}=>
                (first_suffix_field,fields),
            _=>panic!(
                "Attempting to construct a PrefixTypeMetadata from a \
                 TypeLayout of a non-prefix-type.\n\
                 Type:{}\nDataVariant:{:?}\nPackage:{}",
                 layout.full_type,
                 layout.data.discriminant(),
                 layout.package,
            );
        }
        Self{
            fields:fields,
            prefix_field_count:first_suffix_field,
            layout,
        }
    }
}


/// Used to panic with an error message informing the user that a field 
/// is expected to be on the `T` type when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_ty<T>(field_index:usize,actual_layout:&'static TypeLayout)->!
where T:PrefixTypeTrait
{
    panic_on_missing_field_val(field_index,T::layout(),actual_layout)
}


/// Used to panic with an error message informing the user that a field 
/// is expected to be on `expected` when it's not.
#[cold]
#[inline(never)]
pub fn panic_on_missing_field_val(
    field_index:usize,
    expected:&'static TypeLayout,
    actual:&'static TypeLayout,
)->! {
    let expected=PrefixTypeMetadata::new(expected);
    let actual=PrefixTypeMetadata::new(actual);

    let field=expected.layout.fields[field_index];

    panic!("\n
Attempting to access nonexistent field:
    index:{index} 
    named:{field_named}
    type:{field_type}

Type:{struct_type}

Package:'{package}' 

Expected:
    Version(expected compatible):{expected_package_version}
    Field count:{expected_field_count}

Found:
    Version:{expected_package_version}
    Field count:{expected_field_count}

\n",
        index=field_index,
        field_named=field.name.as_str(),
        field_type=field.abi_info.get().full_type,
        struct_type=expected.layout.full_type,
        package=expected.layout.package,
        
        expected_package_version =expected.layout.package_version ,
        expected_field_count=expected.field_count,
        
        actual_package_version =actual.layout.package_version ,
        actual_field_count=actual.field_count,
    );
}