use crate::{
    std_types::RSlice,
    type_layout::{data_structures::ArrayLen, TypeLayout},
};

////////////////////////////////////////////////////////////////////////////////

abi_stable_shared::declare_type_layout_index! {
    attrs=[
        derive(StableAbi),
        sabi(unsafe_sabi_opaque_fields),
    ]
}

////////////////////////////////////////////////////////////////////////////////

abi_stable_shared::declare_multi_tl_types! {
    attrs=[
        derive(StableAbi),
        sabi(unsafe_sabi_opaque_fields),
    ]
}

impl TypeLayoutRange {
    pub(crate) const fn to_array(self) -> [u16; Self::STORED_INLINE] {
        [
            ((self.bits0 >> Self::INDEX_0_OFFSET) & Self::INDEX_MASK) as u16,
            ((self.bits0 >> Self::INDEX_1_OFFSET) & Self::INDEX_MASK) as u16,
            ((self.bits1 >> Self::INDEX_2_OFFSET) & Self::INDEX_MASK) as u16,
            ((self.bits1 >> Self::INDEX_3_OFFSET) & Self::INDEX_MASK) as u16,
            ((self.bits1 >> Self::INDEX_4_OFFSET) & Self::INDEX_MASK) as u16,
        ]
    }

    /// Expands this `TypeLayoutRange` into a `MultipleTypeLayouts<'a>`.
    pub fn expand<'a>(
        &self,
        type_layouts: &'a [extern "C" fn() -> &'static TypeLayout],
    ) -> MultipleTypeLayouts<'a> {
        let indices = self.to_array();
        let len = self.len();

        let first = ArrayLen {
            array: [
                type_layouts[indices[0] as usize],
                type_layouts[indices[1] as usize],
                type_layouts[indices[2] as usize],
                type_layouts[indices[3] as usize],
                type_layouts[indices[4] as usize],
            ],
            len: len.min(Self::STORED_INLINE) as u16,
        };

        let remaining = if len <= Self::STORED_INLINE {
            RSlice::EMPTY
        } else {
            let start_rem = (indices[Self::STORED_INLINE - 1] + 1) as usize;
            let len_rem = len - Self::STORED_INLINE;
            RSlice::from_slice(&type_layouts[start_rem..start_rem + len_rem])
        };

        MultipleTypeLayouts { first, remaining }
    }
}

////////////////////////////////////////////////////////////////////////////////

/// This stores multiple `TypeLayoutCtor`,some inline and some in a borrowed slice.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub struct MultipleTypeLayouts<'a> {
    first: ArrayLen<[extern "C" fn() -> &'static TypeLayout; TypeLayoutRange::STORED_INLINE]>,
    remaining: RSlice<'a, extern "C" fn() -> &'static TypeLayout>,
}

impl<'a> MultipleTypeLayouts<'a> {
    /// The amount of type layouts that this contains.
    pub const fn len(&self) -> usize {
        self.first.len as usize + self.remaining.len()
    }

    /// Whether this is empty.
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets an iterator over the type layouts that this contains.
    pub const fn iter(&self) -> MTLIterator<'a> {
        MTLIterator {
            this: *self,
            index: 0,
        }
    }
}

/// An iterator over a list of type layouts.
#[derive(Clone, Debug)]
pub struct MTLIterator<'a> {
    this: MultipleTypeLayouts<'a>,
    index: usize,
}

impl<'a> Iterator for MTLIterator<'a> {
    type Item = extern "C" fn() -> &'static TypeLayout;

    fn next(&mut self) -> Option<extern "C" fn() -> &'static TypeLayout> {
        if self.index < self.this.len() {
            let ret = if self.index < TypeLayoutRange::STORED_INLINE {
                self.this.first.array[self.index]
            } else {
                self.this.remaining[self.index - TypeLayoutRange::STORED_INLINE]
            };
            self.index += 1;
            Some(ret)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.this.len() - self.index;
        (len, Some(len))
    }
}

impl<'a> ExactSizeIterator for MTLIterator<'a> {}
