use super::*;

use crate::sabi_types::Constructor;

/// The layout of a field.
#[repr(C)]
#[derive(Debug, Copy, Clone, StableAbi)]
#[sabi(unsafe_sabi_opaque_fields)]
pub struct TLField {
    /// The field's name.
    name: RStr<'static>,
    /// Which lifetimes in the struct are referenced in the field type.
    lifetime_indices: LifetimeArrayOrSlice<'static>,
    /// The layout of the field's type.
    ///
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static TypeLayout`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    layout: Constructor<&'static TypeLayout>,

    /// The function pointer types within the field.
    function_range: TLFunctionSlice,

    /// Whether this field is only a function pointer.
    is_function: bool,

    /// How this field is accessed.
    field_accessor: FieldAccessor,
}

///////////////////////////

impl TLField {
    /// Constructs a field which does not contain function pointers,or lifetime indices.
    pub const fn new(
        name: RStr<'static>,
        layout: extern "C" fn() -> &'static TypeLayout,
        vars: &'static SharedVars,
    ) -> Self {
        Self {
            name,
            lifetime_indices: LifetimeArrayOrSlice::EMPTY,
            layout: Constructor(layout),
            function_range: TLFunctionSlice::empty(vars),
            is_function: false,
            field_accessor: FieldAccessor::Direct,
        }
    }

    /// Gets a printable version of the field type.
    pub fn full_type(&self) -> FmtFullType {
        self.layout.get().full_type()
    }

    /// Gets the name of the field
    pub fn name(&self) -> &'static str {
        self.name.as_str()
    }

    /// Gets the lifetimes that the field references.
    pub const fn lifetime_indices(&self) -> LifetimeArrayOrSlice<'static> {
        self.lifetime_indices
    }
    /// Gets the layout of the field type
    pub fn layout(&self) -> &'static TypeLayout {
        self.layout.get()
    }
    /// Gets all the function pointer types in the field.
    pub const fn function_range(&self) -> TLFunctionSlice {
        self.function_range
    }
    /// Gets whether the field is itself a function pointer.
    pub const fn is_function(&self) -> bool {
        self.is_function
    }
    /// Gets the `FieldAccessor` for the type,
    /// which describes whether a field is accessible,and how it is accessed.
    pub const fn field_accessor(&self) -> FieldAccessor {
        self.field_accessor
    }

    /// Used for calling recursive methods,
    /// so as to avoid infinite recursion in types that reference themselves(even indirectly).
    fn recursive<F, U>(self, f: F) -> U
    where
        F: FnOnce(usize, TLFieldShallow) -> U,
    {
        let mut already_recursed = false;
        let mut recursion_depth = !0;
        let mut visited_nodes = !0;

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            recursion_depth = state.recursion_depth;
            visited_nodes = state.visited_nodes;
            state.recursion_depth += 1;
            state.visited_nodes += 1;
            already_recursed = state.visited.replace(self.layout.get()).is_some();
        });

        let _guard = if visited_nodes == 0 {
            Some(ResetRecursion)
        } else {
            None
        };

        let field = TLFieldShallow::new(self, !already_recursed);
        let res = f(recursion_depth, field);

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            state.recursion_depth -= 1;
        });

        res
    }
}

impl Eq for TLField {}

impl PartialEq for TLField {
    fn eq(&self, other: &Self) -> bool {
        self.recursive(|_, this| {
            let r = TLFieldShallow::new(*other, this.layout.is_some());
            this == r
        })
    }
}

impl Display for TLField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout = self.layout.get();
        let (package, version) = layout.item_info().package_and_version();
        writeln!(
            f,
            "field_name:{name}\n\
             type:{ty}\n\
             size:{size} align:{align}\n\
             package:'{package}' version:'{version}'",
            name = self.name,
            ty = layout.full_type(),
            size = layout.size(),
            align = layout.alignment(),
            package = package,
            version = version,
        )?;

        if !self.function_range.is_empty() {
            writeln!(f, "fn pointer(s):")?;
            for func in self.function_range.iter() {
                writeln!(f, "{}", func.to_string().left_padder(4))?;
            }
        }

        if !self.lifetime_indices.is_empty() {
            writeln!(f, "lifetime indices:{:?}", self.lifetime_indices)?;
        }

        Ok(())
    }
}

///////////////////////////

struct ResetRecursion;

impl Drop for ResetRecursion {
    fn drop(&mut self) {
        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            state.recursion_depth = 0;
            state.visited_nodes = 0;
            state.visited.clear();
        });
    }
}

struct RecursionState {
    recursion_depth: usize,
    visited_nodes: u64,
    visited: HashSet<*const TypeLayout>,
}

thread_local! {
    static ALREADY_RECURSED: RefCell<RecursionState> = RefCell::new(RecursionState{
        recursion_depth:0,
        visited_nodes:0,
        visited: HashSet::default(),
    });
}

////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
struct TLFieldShallow {
    name: RStr<'static>,

    full_type: FmtFullType,

    lifetime_indices: LifetimeArrayOrSlice<'static>,

    /// This is None if it already printed that TypeLayout
    layout: Option<&'static TypeLayout>,

    function_range: TLFunctionSlice,

    is_function: bool,

    field_accessor: FieldAccessor,
}

impl TLFieldShallow {
    fn new(field: TLField, include_type_layout: bool) -> Self {
        let layout = field.layout.get();
        TLFieldShallow {
            name: field.name,
            lifetime_indices: field.lifetime_indices,
            layout: if include_type_layout {
                Some(layout)
            } else {
                None
            },
            full_type: layout.full_type(),

            function_range: field.function_range,
            is_function: field.is_function,
            field_accessor: field.field_accessor,
        }
    }
}

////////////////////////////////////

abi_stable_shared::declare_comp_tl_field! {
    attrs=[
        derive(StableAbi),
        sabi(unsafe_sabi_opaque_fields),
    ]
}

impl CompTLField {
    /// Gets the name of the field from `SharedVars`'s string slice.
    pub fn name(&self, strings: &'static str) -> &'static str {
        &strings[self.name_start_len().to_range()]
    }

    /// Gets the name of the field from `SharedVars`'s slice of lifetime indices.
    pub fn lifetime_indices(
        &self,
        indices: &'static [LifetimeIndexPair],
    ) -> LifetimeArrayOrSlice<'static> {
        let comp = LifetimeRange::from_u21(self.lifetime_indices_bits());
        comp.slicing(indices)
    }

    /// Gets the `FieldAccessor` for the type from `SharedVars`'s string slice,
    /// which describes whether a field is accessible,and how it is accessed..
    pub fn field_accessor(&self, strings: &'static str) -> FieldAccessor {
        let name_end = self.name_start_len().end();
        let comp = CompFieldAccessor::from_u3((self.bits0 >> Self::FIELD_ACCESSOR_OFFSET) as u8);
        let accessor_payload = if comp.requires_payload() {
            strings[name_end..].split(';').next().unwrap_or("")
        } else {
            ""
        };
        comp.expand(accessor_payload)
            .unwrap_or(FieldAccessor::Opaque)
    }

    /// Gets the name of the field from `SharedVars`'s slice of type layouts.
    pub const fn type_layout(
        &self,
        type_layouts: &'static [extern "C" fn() -> &'static TypeLayout],
    ) -> extern "C" fn() -> &'static TypeLayout {
        type_layouts[self.type_layout_index()]
    }

    /// Expands this CompTLField into a TLField.
    pub fn expand(
        &self,
        field_index: usize,
        functions: Option<&'static TLFunctions>,
        vars: &'static SharedVars,
    ) -> TLField {
        let strings = vars.strings();
        let function_range = TLFunctionSlice::for_field(field_index, functions, vars);

        TLField {
            name: self.name(strings).into(),
            lifetime_indices: self.lifetime_indices(vars.lifetime_indices()),
            layout: Constructor(self.type_layout(vars.type_layouts())),
            function_range,
            is_function: self.is_function(),
            field_accessor: self.field_accessor(strings),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{abi_stability::stable_abi_trait::get_type_layout, std_types::RString};

    #[test]
    fn roundtrip() {
        const UNIT_CTOR: extern "C" fn() -> &'static TypeLayout = get_type_layout::<()>;
        const U32_CTOR: extern "C" fn() -> &'static TypeLayout = get_type_layout::<u32>;
        const RSTRING_CTOR: extern "C" fn() -> &'static TypeLayout = get_type_layout::<RString>;

        const MONO_VARS: &MonoSharedVars = &MonoSharedVars::new(rstr!("foo;bar; baz; "), rslice![]);

        const VARS: &SharedVars = &SharedVars::new(
            MONO_VARS,
            rslice![UNIT_CTOR, U32_CTOR, RSTRING_CTOR],
            rslice![],
        );

        let vars = VARS;

        let mut arr = [LifetimeIndex::NONE; 5];
        arr[0] = LifetimeIndex::STATIC;
        let lifetime_range = LifetimeRange::from_array(arr);

        let field = CompTLField::new(
            StartLen::new(9, 3),
            lifetime_range,
            CompFieldAccessor::DIRECT,
            TypeLayoutIndex::from_u10(2),
            false,
        );

        assert_eq!(field.name(vars.strings()), "baz",);
        assert_eq!(
            field.lifetime_indices(vars.lifetime_indices()),
            lifetime_range.slicing(vars.lifetime_indices()),
        );
        assert_eq!(field.type_layout(vars.type_layouts()), RSTRING_CTOR,);
        assert_eq!(field.field_accessor(vars.strings()), FieldAccessor::Direct,);
    }
}
