use super::*;

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
};

use core_extensions::SelfOps;

#[cfg(test)]
mod tests;

/// A function which recursively traverses a type layout,
/// calling `callback` for every `TypeLayout` it goes over.
///
fn traverse_type_layouts<'a, F>(layout: &'a TypeLayout, mut callback: F)
where
    F: FnMut(&'a TypeLayout),
{
    let mut state = RecursionState {
        visited: HashSet::new(),
    };

    traverse_type_layouts_inner(layout, &mut state, &mut callback);
}

fn traverse_type_layouts_inner<'a, F>(
    layout: &'a TypeLayout,
    state: &mut RecursionState,
    callback: &mut F,
) where
    F: FnMut(&'a TypeLayout),
{
    if state.visited.replace(layout.get_utypeid()).is_none() {
        callback(layout);

        for nested_layout in layout.shared_vars.type_layouts() {
            traverse_type_layouts_inner(nested_layout(), state, callback);
        }

        if let Some(extra_checks) = layout.extra_checks() {
            for nested_layout in &*extra_checks.nested_type_layouts() {
                traverse_type_layouts_inner(nested_layout, state, callback);
            }
        }
    }
}

struct RecursionState {
    visited: HashSet<UTypeId>,
}

////////////////////////////////////////////////////////////////////////////////

struct DebugState {
    counter: Cell<usize>,
    map: RefCell<HashMap<UTypeId, Option<usize>>>,
    display_stack: RefCell<Vec<UTypeId>>,
    full_type_stack: RefCell<Vec<UTypeId>>,
}

thread_local! {
    static DEBUG_STATE:DebugState=DebugState{
        counter:Cell::new(0),
        map:RefCell::new(HashMap::new()),
        display_stack:RefCell::new(Vec::new()),
        full_type_stack:RefCell::new(Vec::new()),
    };
}

const GET_ERR: &str =
    "Expected DEBUG_STATE.map to contain the UTypeId of all recursive `TypeLayout`s";

impl Debug for TypeLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current_level = !0;
        DEBUG_STATE.with(|state| {
            current_level = state.counter.get();
            if current_level < 1 {
                state.counter.set(current_level + 1);
            }
        });

        if current_level >= 1 {
            let mut index = None;
            DEBUG_STATE.with(|state| {
                index = *state.map.borrow().get(&self.get_utypeid()).expect(GET_ERR);
            });
            let ptr = TypeLayoutPointer {
                key_in_map: index,
                type_: self.full_type(),
            };
            return Debug::fmt(&ptr, f);
        }

        let mut type_infos = Vec::<&'static TypeLayout>::new();

        if current_level == 0 {
            DEBUG_STATE.with(|state| {
                let mut i = 0usize;

                let mut map = state.map.borrow_mut();

                traverse_type_layouts(self, |this| {
                    map.entry(this.get_utypeid())
                        .or_insert(None)
                        .get_or_insert_with(|| {
                            type_infos.push(this);
                            let index = i;
                            i += 1;
                            index
                        });
                });
            })
        }

        // This guard is used to uninitialize the map when returning from this function,
        // even on panics.
        let _guard = DecrementLevel;

        f.debug_struct("TypeLayout")
            .field("name", &self.name())
            .field("full_type", &self.full_type())
            .field("is_nonzero", &self.is_nonzero())
            .field("alignment", &self.alignment())
            .field("size", &self.size())
            .field("data", &self.data())
            .field("extra_checks", &self.extra_checks())
            .field("type_id", &self.get_utypeid())
            .field("item_info", &self.item_info())
            .field("phantom_fields", &self.phantom_fields())
            .field("tag", &self.tag())
            .field("repr_attr", &self.repr_attr())
            .field("mod_refl_mode", &self.mod_refl_mode())
            .observe(|_| drop(_guard))
            .field("nested_type_layouts", &WithIndices(&type_infos))
            .finish()
    }
}

////////////////

const RECURSIVE_INDICATOR: &str = "<{recursive}>";

impl Display for TypeLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut is_cyclic = false;

        DEBUG_STATE.with(|state| {
            let mut stack = state.display_stack.borrow_mut();
            let tid = self.get_utypeid();
            is_cyclic = stack.contains(&tid);
            if !is_cyclic {
                stack.push(tid);
            }
        });

        if is_cyclic {
            write!(f, "{}{}", self.name(), RECURSIVE_INDICATOR)?;
        } else {
            let _guard = DisplayGuard;

            let (package, version) = self.item_info().package_and_version();
            writeln!(
                f,
                "--- Type Layout ---\n\
                 type:{ty}\n\
                 size:{size} align:{align}\n\
                 package:'{package}' version:'{version}'\n\
                 line:{line} mod:{mod_path}",
                ty = self.full_type(),
                size = self.size(),
                align = self.alignment(),
                package = package,
                version = version,
                line = self.item_info().line,
                mod_path = self.item_info().mod_path,
            )?;
            writeln!(f, "data:\n{}", self.data().to_string().left_padder(4))?;
            let phantom_fields = self.phantom_fields();
            if !phantom_fields.is_empty() {
                writeln!(f, "Phantom fields:\n")?;
                for field in phantom_fields {
                    write!(f, "{}", field.to_string().left_padder(4))?;
                }
            }
            writeln!(f, "Tag:\n{}", self.tag().to_string().left_padder(4))?;
            let extra_checks = match self.extra_checks() {
                Some(x) => x.to_string(),
                None => "<nothing>".to_string(),
            };
            writeln!(f, "Extra checks:\n{}", extra_checks.left_padder(4))?;
            writeln!(f, "Repr attribute:{:?}", self.repr_attr())?;
            writeln!(f, "Module reflection mode:{:?}", self.mod_refl_mode())?;
        }

        Ok(())
    }
}

struct DisplayGuard;

impl Drop for DisplayGuard {
    fn drop(&mut self) {
        DEBUG_STATE.with(|state| {
            state.display_stack.borrow_mut().pop();
        });
    }
}

////////////////

impl Display for FmtFullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl Debug for FmtFullType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut is_cyclic = false;

        DEBUG_STATE.with(|state| {
            let mut stack = state.full_type_stack.borrow_mut();
            let tid = self.utypeid;
            is_cyclic = stack.contains(&tid);
            if !is_cyclic {
                stack.push(tid);
            }
        });

        if is_cyclic {
            write!(f, "{}{}", self.name, RECURSIVE_INDICATOR)?;
        } else {
            use self::TLPrimitive as TLP;

            let _guard = FmtFullTypeGuard;

            let (typename, start_gen, before_ty, ty_sep, end_gen) = match self.primitive {
                Some(TLP::SharedRef) => ("&", "", " ", " ", " "),
                Some(TLP::MutRef) => ("&", "", " mut ", " ", " "),
                Some(TLP::ConstPtr) => ("*const", " ", "", " ", " "),
                Some(TLP::MutPtr) => ("*mut", " ", "", " ", " "),
                Some(TLP::Array { .. }) => ("", "[", "", ";", "]"),
                Some(TLP::U8) | Some(TLP::I8) | Some(TLP::U16) | Some(TLP::I16)
                | Some(TLP::U32) | Some(TLP::I32) | Some(TLP::U64) | Some(TLP::I64)
                | Some(TLP::Usize) | Some(TLP::Isize) | Some(TLP::Bool) | Some(TLP::F32)
                | Some(TLP::F64) | None => (self.name, "<", "", ", ", ">"),
            };

            fmt::Display::fmt(typename, f)?;
            let mut is_before_ty = true;
            let generics = self.generics;
            if !generics.is_empty() {
                fmt::Display::fmt(start_gen, f)?;

                let post_iter = |i: usize, len: usize, f: &mut Formatter<'_>| -> fmt::Result {
                    if i + 1 < len {
                        fmt::Display::fmt(ty_sep, f)?;
                    }
                    Ok(())
                };

                let mut i = 0;

                let total_generics_len =
                    generics.lifetime_count() + generics.types.len() + generics.consts.len();

                for param in self.generics.lifetimes() {
                    fmt::Display::fmt(param, &mut *f)?;
                    post_iter(i, total_generics_len, &mut *f)?;
                    i += 1;
                }
                for param in generics.types.iter().cloned() {
                    let layout = param.get();
                    if is_before_ty {
                        fmt::Display::fmt(before_ty, &mut *f)?;
                        is_before_ty = false;
                    }
                    fmt::Debug::fmt(&layout.full_type(), &mut *f)?;
                    post_iter(i, total_generics_len, &mut *f)?;
                    i += 1;
                }
                for param in generics.consts.iter() {
                    fmt::Debug::fmt(param, &mut *f)?;
                    post_iter(i, total_generics_len, &mut *f)?;
                    i += 1;
                }
                fmt::Display::fmt(end_gen, f)?;
            }
        }
        Ok(())
    }
}

struct FmtFullTypeGuard;

impl Drop for FmtFullTypeGuard {
    fn drop(&mut self) {
        DEBUG_STATE.with(|state| {
            state.full_type_stack.borrow_mut().pop();
        });
    }
}

////////////////

struct DecrementLevel;

impl Drop for DecrementLevel {
    fn drop(&mut self) {
        DEBUG_STATE.with(|state| {
            let current = state.counter.get();
            if current == 0 {
                let mut map = state.map.borrow_mut();
                map.clear();
                map.shrink_to_fit();
            }
            state.counter.set(current.saturating_sub(1));
        })
    }
}

////////////////

#[derive(Debug)]
#[allow(dead_code)]
struct TypeLayoutPointer {
    key_in_map: Option<usize>,
    type_: FmtFullType,
}

////////////////

struct WithIndices<'a, T>(&'a [T]);

impl<'a, T> Debug for WithIndices<'a, T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.0.iter().enumerate()).finish()
    }
}
