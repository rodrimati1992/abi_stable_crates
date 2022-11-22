#[doc(hidden)]
#[macro_export]
macro_rules! declare_comp_field_accessor {(
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (


    /// A compressed field accessor,represented as 3 bits inside of a CompTLField.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct CompFieldAccessor(u8);

    /// The type that CompFieldAccessor is represented as.
    pub type CompFieldAccessorRepr=u8;

    impl CompFieldAccessor{
        /// Equivalent to the `FieldAccessor::Direct` variant.
        pub const DIRECT:Self=CompFieldAccessor(0);
        /// Equivalent to the `FieldAccessor::Method` variant.
        pub const METHOD:Self=CompFieldAccessor(1);
        /// Equivalent to the `FieldAccessor::MethodNamed` variant,
        /// in which the name is stored within SharedVars after the
        /// name of the field this is an accessor for.
        pub const METHOD_NAMED:Self=CompFieldAccessor(2);
        /// Equivalent to the `FieldAccessor::MethodOption` variant.
        pub const METHOD_OPTION:Self=CompFieldAccessor(3);
        /// Equivalent to the `FieldAccessor::Opaque` variant.
        pub const OPAQUE:Self=CompFieldAccessor(4);
    }


    impl CompFieldAccessor{
        const MASK:u8=0b111;
        /// The amount of bits used to represent a CompFieldAccessor.
        pub const BIT_SIZE:u32=3;

        /// Converts this `CompFieldAccessor` into its representation.
        pub const fn to_u3(self)->u8{
            self.0&Self::MASK
        }

        /// Constructs this `CompFieldAccessor` from its representation.
        pub const fn from_u3(n:u8)->Self{
            CompFieldAccessor(n&Self::MASK)
        }
        pub(crate) const fn requires_payload(self)->bool{
            matches!(self, Self::METHOD_NAMED)
        }
    }
)}
