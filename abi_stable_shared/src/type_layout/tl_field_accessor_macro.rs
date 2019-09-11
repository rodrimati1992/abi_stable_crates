#[doc(hidden)]
#[macro_export]
macro_rules! declare_comp_field_accessor {( 
    attrs=[ $($extra_attrs:meta),* $(,)* ]
) => (

    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    $(#[ $extra_attrs ])*
    pub struct CompFieldAccessor(u8);
    
    pub type CompFieldAccessorRepr=u8;

    impl CompFieldAccessor{
        pub const DIRECT:Self=CompFieldAccessor(0);
        pub const METHOD:Self=CompFieldAccessor(1);
        pub const METHOD_NAMED:Self=CompFieldAccessor(2);
        pub const METHOD_OPTION:Self=CompFieldAccessor(3);
        pub const OPAQUE:Self=CompFieldAccessor(4);
    }


    impl CompFieldAccessor{
        pub const MASK:u8=0b111;
        pub const BIT_SIZE:u32=3;

        pub const fn to_u3(self)->u8{
            self.0&Self::MASK
        }
        pub const fn from_u3(n:u8)->Self{
            CompFieldAccessor(n&Self::MASK)
        }
        pub fn requires_payload(self)->bool{
            core_extensions::matches!( Self::METHOD_NAMED= self )
        }
    }
)}