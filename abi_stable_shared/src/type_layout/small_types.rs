#[doc(hidden)]
#[macro_export]
macro_rules! declare_start_len_bit_methods {
    () => {
        /// The amount of bits used to represent a StartLen
        pub const BIT_SIZE: u32 = 26;
        const START_SR_MASK: u32 = 0xFFFF;
        const LEN_SR_MASK: u32 = 0b11_1111_1111;
        pub(crate) const IDENT_MAX_LEN: u16 = Self::LEN_SR_MASK as u16;
        const LEN_OFFSET: u32 = 16;

        /// The exclusive end of this range.
        #[inline]
        pub const fn end(self) -> usize {
            (self.start() + self.len()) as usize
        }

        /// Constructs a StartLen from a bitfield encoded as
        /// (start:16 bits) + (length: 10 bits) .
        pub const fn from_u26(n: u32) -> Self {
            Self::new(
                (n & Self::START_SR_MASK) as _,
                ((n >> Self::LEN_OFFSET) & Self::LEN_SR_MASK) as _,
            )
        }

        /// Converts this StartLen to bitfields encoded as
        /// (start:16 bits) + (length: 10 bits) .
        pub const fn to_u26(self) -> u32 {
            (self.start() as u32) & Self::START_SR_MASK
                | ((self.len() as u32 & Self::LEN_SR_MASK) << Self::LEN_OFFSET)
        }
    };
}
