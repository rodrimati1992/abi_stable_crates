#[doc(hidden)]
#[macro_export]
macro_rules! declare_start_len_bit_methods {() => (

    pub const BIT_SIZE:u32=26;
    pub const START_SR_MASK:u32=0xFFFF;
    pub const LEN_SR_MASK:u32=0b11_1111_1111;
    pub const IDENT_MAX_LEN:u16=Self::LEN_SR_MASK as u16;
    pub const LEN_OFFSET:u32=16;

    /// The start of this range.
    #[inline]
    pub const fn start(self)->usize{
        self.start as usize
    }

    /// The length of this range.
    #[inline]
    pub const fn len(self)->usize{
        self.len as usize
    }

    /// The exclusive end of this range.
    #[inline]
    pub const fn end(self)->usize{
        (self.start+self.len) as usize
    }
    /// Converts this StartLen to bitfields encoded as
    /// (start:16 bits) + (length: 10 bits) . 
    pub const fn to_u26(self)->u32{
         (self.start as u32)&Self::START_SR_MASK
        |((self.len as u32 & Self::LEN_SR_MASK ) << Self::LEN_OFFSET)
    }

    /// Constructs a StartLen from a bitfield encoded as
    /// (start:16 bits) + (length: 10 bits) . 
    pub const fn from_u26(n:u32)->Self{
        Self{
            start:(n & Self::START_SR_MASK) as u16,
            len:((n>>Self::LEN_OFFSET)& Self::LEN_SR_MASK)as u16,
        }
    }

)}