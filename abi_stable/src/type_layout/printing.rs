use super::*;

use std::{
    collections::{HashSet,HashMap},
    cell::{Cell,RefCell},
};

use core_extensions::SelfOps;

/**
A function which recursively traverses a type layout,
calling `callback` for every `TypeLayout` it goes over.

*/
fn traverse_type_layouts<'a,F>(layout:&'a TypeLayout,mut callback:F)
where
    F:FnMut(&'a TypeLayout)
{
    let mut state=RecursionState{
        visited:HashSet::new(),
    };

    traverse_type_layouts_inner(
        layout,
        &mut state,
        &mut callback,
    );
}


fn traverse_type_layouts_inner<'a,F>(
    layout:&'a TypeLayout,
    state:&mut RecursionState,
    callback:&mut F,
)where
    F:FnMut(&'a TypeLayout)
{
    if state.visited.replace(layout.get_utypeid()).is_none() {
        callback(layout);

        // This is inlined here because the proper way to access all `&'a TypeLayout`
        // that a type contains requires 
        let fields=match layout.data {
            TLData::Primitive{..}|TLData::Opaque=>
                TLFieldsOrSlice::EMPTY,
            TLData::Struct{fields}=>fields,
            TLData::Union{fields}=>fields,
            TLData::Enum (tlenum)=>tlenum.fields,
            TLData::PrefixType(prefix)=>prefix.fields,
        };

        if let TLFieldsOrSlice::TLFields(TLFields{functions:Some(functions),..})=fields {
            for get_type_layout in functions.type_layouts.as_slice() {
                traverse_type_layouts_inner(get_type_layout.get(), state, callback);
            }
        }


        for field in fields.get_fields() {
            traverse_type_layouts_inner(field.layout.get(), state, callback);
        }

        for field in layout.phantom_fields.as_slice() {
            traverse_type_layouts_inner(field.layout.get(), state, callback);
        }

        if let Some(extra_checks)=layout.extra_checks() {
            for layout in &*extra_checks.nested_type_layouts() {
                traverse_type_layouts_inner(layout, state, callback);
            }
        }
    }
}



struct RecursionState{
    visited:HashSet<UTypeId>,
}


////////////////////////////////////////////////////////////////////////////////

struct DebugState{
    counter:Cell<usize>,
    map:RefCell<HashMap<UTypeId,usize>>,
}


thread_local!{
    static DEBUG_STATE:DebugState=DebugState{
        counter:Cell::new(0),
        map:RefCell::new(HashMap::new()),
    };
}

const GET_ERR:&'static str=
    "Expected DEBUG_STATE.map to contain the UTypeId of all recursive `TypeLayout`s";

impl Debug for TypeLayout{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        let mut current_level=!0;
        DEBUG_STATE.with(|state|{
            current_level=state.counter.get();
            if current_level < 1 {
                state.counter.set(current_level+1);
            }
        });

        if current_level>=1 {
            let mut index=0;
            DEBUG_STATE.with(|state|{
                index=*state.map.borrow().get(&self.get_utypeid()).expect(GET_ERR) ;
            });
            let ptr=TypeLayoutPointer{
                key_in_map:index,
                type_:&self.full_type,
            };
            return Debug::fmt(&ptr,f);
        }

        let mut type_infos=Vec::<&'static TypeLayout>::new();

        if current_level==0 {
            DEBUG_STATE.with(|state|{
                let mut i=0usize;

                let mut map=state.map.borrow_mut();
                
                traverse_type_layouts(self,|this|{
                    map.entry(this.get_utypeid())
                        .or_insert_with(||{ 
                            type_infos.push(this);
                            let index=i;
                            i+=1; 
                            index
                        });
                });
            })
        }



        // This guard is used to uninitialize the map when returning from this function,
        // even on panics.
        let _guard=DecrementLevel;

        f.debug_struct("TypeLayout")
            .field("full_type",&self.full_type)
            .field("name",&self.name)
            .field("abi_consts",&self.abi_consts)
            .field("item_info",&self.item_info)
            .field("size",&self.size)
            .field("alignment",&self.alignment)
            .field("data",&self.data)
            .field("phantom_fields",&self.phantom_fields)
            .field("reflection_tag",&self.reflection_tag)
            .field("tag",&self.tag)
            .field("extra_checks",&self.extra_checks())
            .field("repr_attr",&self.repr_attr)
            .field("mod_refl_mode",&self.mod_refl_mode)
            .observe(|_| drop(_guard) )
            .field("nested_type_layouts",&WithIndices(&type_infos))
            .finish()
    }
}


////////////////


struct DecrementLevel;

impl Drop for DecrementLevel{
    fn drop(&mut self){
        DEBUG_STATE.with(|state|{
            let current=state.counter.get();
            if current==0 {
                let mut map=state.map.borrow_mut();
                map.clear();
                map.shrink_to_fit();
            }
            state.counter.set(current.saturating_sub(1));
        })
    }
}



////////////////

#[derive(Debug)]
struct TypeLayoutPointer<'a>{
    key_in_map:usize,
    type_:&'a FullType,
}


////////////////


struct WithIndices<'a,T>(&'a [T]);

impl<'a,T> Debug for WithIndices<'a,T>
where
    T:Debug
{
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{
        f.debug_map()
         .entries(self.0.iter().enumerate())
         .finish()
    }
}