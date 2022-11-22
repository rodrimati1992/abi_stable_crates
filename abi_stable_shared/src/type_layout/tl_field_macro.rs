#[doc(hidden)]
#[macro_export]
macro_rules! declare_comp_tl_field {(
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    /// A `TLField` represented as a `u64`,
    /// expadable to a `TLField` by calling the `expand` method.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    $(#[ $extra_attrs ])*
    pub struct CompTLField{
        bits0:u64,
    }

    /// The underlying representation of `CompTLField`.
    pub type CompTLFieldRepr=u64;

    impl CompTLField{
        const NAME_OFFSET:u32=0;

        const LIFETIME_INDICES_OFFSET:u32=StartLen::BIT_SIZE;

        const FIELD_ACCESSOR_OFFSET:u32=Self::LIFETIME_INDICES_OFFSET+LifetimeRange::BIT_SIZE;

        const TYPE_LAYOUT_OFFSET:u32=Self::FIELD_ACCESSOR_OFFSET+CompFieldAccessor::BIT_SIZE;
        const TYPE_LAYOUT_SR_MASK:u64=TypeLayoutIndex::MASK as u64;

        const IS_FUNCTION_OFFSET:u32=Self::TYPE_LAYOUT_OFFSET+TypeLayoutIndex::BIT_SIZE;
        const IS_FUNCTION_BIT_SIZE:u32=1;

        /// The amount of bits necessary to represent a CompTLField.
        pub const BIT_SIZE:u32=Self::IS_FUNCTION_OFFSET+Self::IS_FUNCTION_BIT_SIZE;

        /// Constructs a CompTLField.
        #[inline]
        pub const fn new(
            name:StartLen,
            lifetime_indices:LifetimeRange,
            field_accessor:CompFieldAccessor,
            layout: TypeLayoutIndex,
            is_function:bool,
        )->Self{
            // A compile-time assertion that the bit fields fit inside a u64
            let _:[(); (64-Self::BIT_SIZE)as usize];

            let bits0={
                 ((name.to_u26() as u64)<<Self::NAME_OFFSET)
                |((lifetime_indices.to_u21() as u64)<<Self::LIFETIME_INDICES_OFFSET)
                |((field_accessor.to_u3() as u64)<<Self::FIELD_ACCESSOR_OFFSET)
                |((layout.to_u10() as u64)<<Self::TYPE_LAYOUT_OFFSET)
                |((is_function as u64)<<Self::IS_FUNCTION_OFFSET)
            };

            CompTLField{bits0}
        }

        /// Gets the range representing the name in the string slice field inside
        /// the `SharedVars` field of the `TypeLayout` that contains this.
        #[inline]
        pub const fn name_start_len(&self)->StartLen{
            StartLen::from_u26((self.bits0>>Self::NAME_OFFSET) as u32)
        }

        /// Gets the index of the type layout of the field in
        /// the slice of type layouts inside
        /// the `SharedVars` field of the `TypeLayout` that contains this.
        #[inline]
        pub const fn type_layout_index(&self)-> usize {
            ((self.bits0>>Self::TYPE_LAYOUT_OFFSET)&Self::TYPE_LAYOUT_SR_MASK)as usize
        }


        #[inline]
        const fn lifetime_indices_bits(&self)-> u32 {
            (self.bits0>>Self::LIFETIME_INDICES_OFFSET)as u32
        }

        /// Whether this field is a function.
        /// This is only true if the type is a function pointer(not inside some other type).
        #[inline]
        pub const fn is_function(&self)->bool{
            (self.bits0 & (1<<Self::IS_FUNCTION_OFFSET))!=0
        }

        #[inline]
        pub(crate) const fn std_field(
            name:StartLen,
            lifetime_indices:LifetimeRange,
            layout: u16,
        )->Self{
            Self::new(
                name,
                lifetime_indices,
                CompFieldAccessor::DIRECT,
                TypeLayoutIndex::from_u10(layout),
                false,
            )
        }
    }
)}
