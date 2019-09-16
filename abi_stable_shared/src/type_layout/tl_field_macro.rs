#[doc(hidden)]
#[macro_export]
macro_rules! declare_comp_tl_field {( 
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    #[repr(transparent)]
    #[derive(Debug, Copy, Clone)]
    $(#[ $extra_attrs ])*
    pub struct CompTLField{
        bits0:u64,
    }

    pub type CompTLFieldRepr=u64;

    impl CompTLField{
        const NAME_OFFSET:u32=0;
        
        const LIFETIME_INDICES_OFFSET:u32=StartLen::BIT_SIZE;
        
        const FIELD_ACCESSOR_OFFSET:u32=Self::LIFETIME_INDICES_OFFSET+LifetimeRange::BIT_SIZE;
        
        const TYPE_LAYOUT_OFFSET:u32=Self::FIELD_ACCESSOR_OFFSET+CompFieldAccessor::BIT_SIZE;
        const TYPE_LAYOUT_SR_MASK:u64=TypeLayoutIndex::MASK as u64;
        
        pub const TYPE_LAYOUT_MAX_VAL:usize=TypeLayoutIndex::MASK as usize;

        const IS_FUNCTION_OFFSET:u32=Self::TYPE_LAYOUT_OFFSET+TypeLayoutIndex::BIT_SIZE;
        const IS_FUNCTION_BIT_SIZE:u32=1;

        pub const BIT_SIZE:u32=Self::IS_FUNCTION_OFFSET+Self::IS_FUNCTION_BIT_SIZE;

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

        #[inline]
        pub fn name_start_len(&self)->StartLen{
            StartLen::from_u26((self.bits0>>Self::NAME_OFFSET) as u32)
        }

        #[inline]
        pub fn type_layout_index(&self)-> usize {
            ((self.bits0>>Self::TYPE_LAYOUT_OFFSET)&Self::TYPE_LAYOUT_SR_MASK)as usize
        }

        #[inline]
        pub fn lifetime_indices_bits(&self)-> u32 {
            (self.bits0>>Self::LIFETIME_INDICES_OFFSET)as u32
        }

        #[inline]
        pub fn is_function(&self)->bool{
            (self.bits0 & (1<<Self::IS_FUNCTION_OFFSET))!=0
        }

        #[inline]
        pub const fn std_field(
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