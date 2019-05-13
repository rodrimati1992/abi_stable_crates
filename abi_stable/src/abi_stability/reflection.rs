/// Whether this is a module whose definition can be reflected on at runtime,
#[repr(C)]
#[derive(Debug,Copy,Clone,PartialEq,Eq,StableAbi)]
pub enum ModReflMode{
    /// For modules that are reflected on at runtime..
    Module,
    /// For types whose layout can't be iterated over.
    Opaque,
    /// Delegates the layout to some other type,this is generally for references.
    Delegate,
}


impl Default for ModReflMode{
    fn default()->Self{
        ModReflMode::Opaque
    }
}
