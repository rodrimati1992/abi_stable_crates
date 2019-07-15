use super::*;


/// The layout of a field.
#[repr(C)]
#[derive(Copy, Clone,StableAbi)]
pub struct TLField {
    /// The field's name.
    pub name: StaticStr,
    /// Which lifetimes in the struct are referenced in the field type.
    pub lifetime_indices: StaticSlice<LifetimeIndex>,
    /// The layout of the field's type.
    ///
    /// This is a function pointer to avoid infinite recursion,
    /// if you have a `&'static AbiInfo`s with the same address as one of its parent type,
    /// you've encountered a cycle.
    pub abi_info: GetAbiInfo,

    pub function_range:TLFunctionRange,

    /// Whether this field is only a function pointer.
    pub is_function:bool,

    pub field_accessor:FieldAccessor,
}


/// Whether a field is accessible,and how it is accessed.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, StableAbi)]
pub enum FieldAccessor {
    /// Accessible with `self.field_name`
    Direct,
    /// Accessible with `fn field_name(&self)->FieldType`
    Method{
        name:Option<&'static StaticStr>,
    },
    /// Accessible with `fn field_name(&self)->Option<FieldType>`
    MethodOption,
    /// This field is completely inaccessible.
    Opaque,
}


impl FieldAccessor{
    pub const fn method_named(name:&'static StaticStr)->Self{
        FieldAccessor::Method{
            name:Some(name)
        }
    }
}


///////////////////////////

impl TLField {
    /// Constructs a field which does not contain function pointers.
    pub const fn new(
        name: &'static str,
        lifetime_indices: &'static [LifetimeIndex],
        abi_info: GetAbiInfo,
    ) -> Self {
        Self {
            name: StaticStr::new(name),
            lifetime_indices: StaticSlice::new(lifetime_indices),
            abi_info,
            function_range:TLFunctionRange::EMPTY,
            is_function:false,
            field_accessor:FieldAccessor::Direct,
        }
    }

    pub const fn set_field_accessor(mut self,field_accessor:FieldAccessor)->Self{
        self.field_accessor=field_accessor;
        self
    }


    pub fn full_type(&self)->FullType{
        self.abi_info.get().layout.full_type
    }


    /// Used for calling recursive methods,
    /// so as to avoid infinite recursion in types that reference themselves(even indirectly).
    fn recursive<F, U>(self, f: F) -> U
    where
        F: FnOnce(usize,TLFieldShallow) -> U,
    {
        let mut already_recursed = false;
        let mut recursion_depth=!0;
        let mut visited_nodes=!0;

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            recursion_depth=state.recursion_depth;
            visited_nodes=state.visited_nodes;
            state.recursion_depth+=1;
            state.visited_nodes+=1;
            already_recursed = state.visited.replace(self.abi_info.get()).is_some();
        });

        let _guard=if visited_nodes==0 { Some(ResetRecursion) }else{ None };

        let field=TLFieldShallow::new(self, !already_recursed );
        let res = f( recursion_depth, field);

        ALREADY_RECURSED.with(|state| {
            let mut state = state.borrow_mut();
            state.recursion_depth-=1;
        });

        res
    }
}

impl Eq for TLField{}

impl PartialEq for TLField {
    fn eq(&self, other: &Self) -> bool {
        self.recursive(|_,this| {
            let r = TLFieldShallow::new(*other, this.abi_info.is_some());
            this == r
        })
    }
}

/// Need to avoid recursion somewhere,so I decided to stop at the field level.
impl Debug for TLField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.recursive(|recursion_depth,x|{
            // if recursion_depth>=5 {
            //     writeln!(f,"<printing recursion limit>")
            // }else{
                fmt::Debug::fmt(&x, f)
            // }
        })
    }
}

impl Display for TLField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let layout=self.abi_info.get().layout;
        let (package,version)=layout.item_info.package_and_version();
        writeln!(
            f,
            "field_name:{name}\n\
             type:{ty}\n\
             size:{size} align:{align}\n\
             package:'{package}' version:'{version}'",
            name =self.name,
            ty   =layout.full_type(),
            size =layout.size,
            align=layout.alignment,
            package=package,
            version=version,
        )?;

        if !self.function_range.is_empty() {
            writeln!(f,"fn pointer(s):")?;
            for func in self.function_range.iter() {
                writeln!(f,"{}",func.to_string().left_padder(4))?;
            }
        }

        if !self.lifetime_indices.is_empty() {
            writeln!(f,"lifetime indices:{:?}",self.lifetime_indices)?;
        }

        Ok(())
    }
}




///////////////////////////


struct ResetRecursion;

impl Drop for ResetRecursion{
    fn drop(&mut self){
        ALREADY_RECURSED.with(|state|{
            let mut state = state.borrow_mut();
            state.recursion_depth=0;
            state.visited_nodes=0;
            state.visited.clear();
        });
    }
}


struct RecursionState{
    recursion_depth:usize,
    visited_nodes:u64,
    visited:HashSet<*const AbiInfo>,
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
    pub(crate) name: StaticStr,
    pub(crate) full_type: FullType,
    pub(crate) lifetime_indices: StaticSlice<LifetimeIndex>,
    /// This is None if it already printed that AbiInfo
    pub(crate) abi_info: Option<&'static AbiInfo>,

    pub(crate) function_range:TLFunctionRange,

    pub(crate) is_function:bool,

    pub(crate) field_accessor:FieldAccessor,
}

impl TLFieldShallow {
    fn new(field: TLField, include_abi_info: bool) -> Self {
        let abi_info = field.abi_info.get();
        TLFieldShallow {
            name: field.name,
            lifetime_indices: field.lifetime_indices,
            abi_info: if include_abi_info {
                Some(abi_info)
            } else {
                None
            },
            full_type: abi_info.layout.full_type,

            function_range:field.function_range,
            is_function:field.is_function,
            field_accessor:field.field_accessor,
        }
    }
}

