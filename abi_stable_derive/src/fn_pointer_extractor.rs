//! Contains visitor type for
//! extracting function pointers and the referenced lifetimes of the fields of a type declaration.

use std::{collections::HashSet, mem};

use as_derive_utils::{spanned_err, syn_err};

use core_extensions::SelfOps;

use syn::{
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Generics, Ident, Lifetime, Type, TypeBareFn, TypeReference,
};

use proc_macro2::Span;

use quote::ToTokens;

use crate::*;
use crate::{
    common_tokens::FnPointerTokens,
    ignored_wrapper::Ignored,
    lifetimes::{LifetimeCounters, LifetimeIndex},
    utils::{LinearResult, SynResultExt},
};

/// Information about all the function pointers in a type declaration.
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct FnInfo<'a> {
    /// The generics of the struct this function pointer type is used inside of.
    parent_generics: &'a Generics,

    /// The identifiers for all the lifetimes of the
    /// struct this function pointer type is used inside of
    env_lifetimes: Vec<&'a Ident>,

    /// The index of first lifetime declared by all functions.
    /// (with higher lifetime indices from the struct/enum definition it is used inside of).
    initial_bound_lifetime: usize,

    pub functions: Vec<Function<'a>>,
}

/// A function pointer in a type declaration.
#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct Function<'a> {
    pub(crate) fn_token: syn::Token!(fn),
    pub(crate) func_span: Ignored<Span>,

    /// The index of the first lifetime the function declares,if there are any.
    pub(crate) first_bound_lt: usize,

    /// The index of the first unnamed lifetime of the function,if there are any.
    pub(crate) first_unnamed_bound_lt: usize,

    /// The named lifetimes for this function pointer type,
    /// the ones declared within `for<'a,'b,'c>`.
    pub(crate) named_bound_lts: Vec<&'a Ident>,
    /// A set version of the `named_bound_lts` field.
    pub(crate) named_bound_lt_set: Ignored<HashSet<&'a Ident>>,
    /// The amount of lifetimes declared by the function pointer.
    pub(crate) bound_lts_count: usize,

    pub(crate) is_unsafe: bool,

    /// The Span for the first time that a bound lifetime appears in the type definition.
    pub(crate) bound_lt_spans: Ignored<Vec<Option<Span>>>,

    /// The parameters of this function pointer,including name and type.
    pub(crate) params: Vec<FnParamRet<'a>>,
    /// What this function pointer returns,including name and type.
    ///
    /// None if its return type is `()`.
    pub(crate) returns: Option<FnParamRet<'a>>,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub(crate) struct FnParamRet<'a> {
    /// The name of the parameter/return type.
    ///
    /// This is None if the parameter doesn't have a name.
    pub(crate) name: Option<&'a Ident>,
    /// The lifetimes this type references (including static).
    pub(crate) lifetime_refs: Vec<LifetimeIndex>,
    /// The type of the parameter/return type.
    pub(crate) ty: &'a Type,
    /// Whether this is a parameter or a return type.
    pub(crate) param_or_ret: ParamOrReturn,
}

/// The information returned from visiting a field.
pub(crate) struct VisitFieldRet<'a> {
    /// The lifetimes that the field references.
    pub(crate) referenced_lifetimes: Vec<LifetimeIndex>,
    /// The function pointer types in the field.
    pub(crate) functions: Vec<Function<'a>>,
}

/////////////

#[allow(dead_code)]
impl<'a> TypeVisitor<'a> {
    #[inline(never)]
    pub fn new(arenas: &'a Arenas, ctokens: &'a FnPointerTokens, generics: &'a Generics) -> Self {
        TypeVisitor {
            refs: ImmutableRefs {
                arenas,
                ctokens,
                env_generics: generics,
            },
            vars: Vars {
                allow_type_macros: false,
                referenced_lifetimes: Vec::default(),
                fn_info: FnInfo {
                    parent_generics: generics,
                    env_lifetimes: generics.lifetimes().map(|lt| &lt.lifetime.ident).collect(),
                    initial_bound_lifetime: generics.lifetimes().count(),
                    functions: Vec::new(),
                },
                errors: LinearResult::ok(()),
            },
        }
    }

    pub fn allow_type_macros(&mut self) {
        self.vars.allow_type_macros = true;
    }

    /// Gets the arena this references.
    pub fn arenas(&self) -> &'a Arenas {
        self.refs.arenas
    }
    /// Gets the CommonTokens this references.
    pub fn ctokens(&self) -> &'a FnPointerTokens {
        self.refs.ctokens
    }
    /// Gets the generic parameters this references.
    pub fn env_generics(&self) -> &'a Generics {
        self.refs.env_generics
    }

    /// Visit a field type,
    /// returning the function pointer types and referenced lifetimes.
    pub fn visit_field(&mut self, ty: &mut Type) -> VisitFieldRet<'a> {
        self.visit_type_mut(ty);
        VisitFieldRet {
            referenced_lifetimes: mem::take(&mut self.vars.referenced_lifetimes),
            functions: mem::take(&mut self.vars.fn_info.functions),
        }
    }

    pub fn get_errors(&mut self) -> Result<(), syn::Error> {
        self.vars.errors.take()
    }

    pub fn into_fn_info(self) -> FnInfo<'a> {
        self.vars.fn_info
    }
}

/////////////

/// Whether this is a parameter or a return type/value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ParamOrReturn {
    Param,
    Return,
}

/// A type which visits an entire type definition a field at a time,
/// extracting function pointers and lifetimes for each field.
pub(crate) struct TypeVisitor<'a> {
    /// immutable references shared with other data structures-
    refs: ImmutableRefs<'a>,
    /// variables which are mutated when visiting.
    vars: Vars<'a>,
}

/// Some immutable references used when visiting field types.
#[allow(dead_code)]
#[derive(Copy, Clone)]
struct ImmutableRefs<'a> {
    arenas: &'a Arenas,
    ctokens: &'a FnPointerTokens,
    /// Generics provided by the environment (eg:the struct this type is used inside of).
    env_generics: &'a Generics,
}

/// variables which are mutated when visiting.
struct Vars<'a> {
    allow_type_macros: bool,
    /// What lifetimes in env_lifetimes are referenced in the type being visited.
    /// For TLField.
    referenced_lifetimes: Vec<LifetimeIndex>,
    fn_info: FnInfo<'a>,
    errors: LinearResult<()>,
}

/// Used to visit a function pointer type.
struct FnVisitor<'a, 'b> {
    refs: ImmutableRefs<'a>,
    vars: &'b mut Vars<'a>,

    lifetime_counts: LifetimeCounters,

    /// The current function pointer type that is being visited,
    current: Function<'a>,
    /// The lifetime indices inside a parameter/return type that is currently being visited.
    param_ret: FnParamRetLifetimes,
}

/// The lifetime indices inside a parameter/return type.
struct FnParamRetLifetimes {
    span: Span,
    /// The lifetimes this type references (including static).
    lifetime_refs: Vec<LifetimeIndex>,
    /// Whether this is a parameter or return type.
    param_or_ret: ParamOrReturn,
}

/////////////

impl FnParamRetLifetimes {
    fn new(param_or_ret: ParamOrReturn, span: Option<Span>) -> Self {
        Self {
            span: span.unwrap_or_else(Span::call_site),
            lifetime_refs: Vec::new(),
            param_or_ret,
        }
    }
}

/////////////

impl<'a> Vars<'a> {
    /// Registers a lifetime index,
    /// selecting those that come from the type declaration itself.
    pub fn add_referenced_env_lifetime(&mut self, ind: LifetimeIndex) {
        let is_env_lt = match (ind, ind.to_param()) {
            (LifetimeIndex::STATIC, _) => true,
            (_, Some(index)) => index < self.fn_info.env_lifetimes.len(),
            _ => false,
        };
        if is_env_lt {
            self.referenced_lifetimes.push(ind);
        }
    }
}

/////////////

impl<'a> VisitMut for TypeVisitor<'a> {
    /// Visits a function pointer type within a field type.
    #[inline(never)]
    fn visit_type_bare_fn_mut(&mut self, func: &mut TypeBareFn) {
        let ctokens = self.refs.ctokens;
        let arenas = self.refs.arenas;

        let func_span = func.span();

        let is_unsafe = func.unsafety.is_some();

        let abi = func.abi.as_ref().map(|x| x.name.as_ref());
        const ABI_ERR: &str = "must write `extern \"C\" fn` for function pointer types.";
        match abi {
            Some(Some(abi)) if *abi == ctokens.c_abi_lit => {}
            Some(Some(abi)) => {
                self.vars
                    .errors
                    .push_err(spanned_err!(abi, "Abi not supported for function pointers",));
                return;
            }
            Some(None) => {}
            None => {
                self.vars.errors.push_err(spanned_err!(
                    func,
                    "The default abi is not supported for function pointers,you {}`.",
                    ABI_ERR
                ));
                return;
            }
        }

        let named_bound_lts: Vec<&'a Ident> = func
            .lifetimes
            .take() // Option<BoundLifetimes>
            .into_iter()
            .flat_map(|lt| lt.lifetimes)
            .map(|lt| arenas.alloc(lt.lifetime.ident))
            .collect::<Vec<&'a Ident>>();

        let named_bound_lt_set = named_bound_lts.iter().cloned().collect();

        let first_bound_lt = self.vars.fn_info.initial_bound_lifetime;
        let bound_lts_count = named_bound_lts.len();
        let mut current_function = FnVisitor {
            refs: self.refs,
            vars: &mut self.vars,
            lifetime_counts: LifetimeCounters::new(),
            current: Function {
                fn_token: func.fn_token,
                func_span: Ignored::new(func_span),
                first_bound_lt,
                first_unnamed_bound_lt: first_bound_lt + named_bound_lts.len(),
                bound_lts_count,
                named_bound_lts,
                named_bound_lt_set: Ignored::new(named_bound_lt_set),
                bound_lt_spans: Ignored::new(vec![None; bound_lts_count]),
                is_unsafe,
                params: Vec::new(),
                returns: None,
            },
            param_ret: FnParamRetLifetimes::new(ParamOrReturn::Param, None),
        };

        // Visits a parameter or return type within a function pointer type.
        fn visit_param_ret<'a, 'b>(
            this: &mut FnVisitor<'a, 'b>,
            name: Option<&'a Ident>,
            ty: &'a mut Type,
            param_or_ret: ParamOrReturn,
        ) {
            let ty_span = Some(ty.span());

            this.param_ret = FnParamRetLifetimes::new(param_or_ret, ty_span);

            this.visit_type_mut(ty);

            let param_ret = mem::replace(
                &mut this.param_ret,
                FnParamRetLifetimes::new(param_or_ret, ty_span),
            );

            let param_ret = FnParamRet {
                name,
                lifetime_refs: param_ret.lifetime_refs,
                ty,
                param_or_ret: param_ret.param_or_ret,
            };

            match param_or_ret {
                ParamOrReturn::Param => this.current.params.push(param_ret),
                ParamOrReturn::Return => this.current.returns = Some(param_ret),
            }
        }

        for (i, param) in func.inputs.iter_mut().enumerate() {
            let arg_name = extract_fn_arg_name(i, param, arenas);
            let ty = arenas.alloc_mut(param.ty.clone());
            visit_param_ret(&mut current_function, arg_name, ty, ParamOrReturn::Param);
        }

        let tmp = match mem::replace(&mut func.output, syn::ReturnType::Default) {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(arenas.alloc_mut(*ty)),
        };
        if let Some(ty) = tmp {
            visit_param_ret(&mut current_function, None, ty, ParamOrReturn::Return);
        }

        let mut current = current_function.current;
        current.anonimize_lifetimes(&current_function.lifetime_counts, &mut self.vars.errors);
        while func.inputs.pop().is_some() {}
        self.vars.fn_info.functions.push(current);
    }

    /// Visits a lifetime within a field type,
    /// pushing it to the list of referenced lifetimes.
    #[inline(never)]
    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        let ctokens = self.refs.ctokens;
        let lt = &lt.ident;
        if *lt == ctokens.static_ {
            LifetimeIndex::STATIC
        } else {
            let env_lifetimes = self.vars.fn_info.env_lifetimes.iter();
            let found_lt = env_lifetimes
                .enumerate()
                .position(|(_, lt_ident)| *lt_ident == lt);
            match found_lt {
                Some(index) => LifetimeIndex::Param(index as _),
                None => {
                    self.vars
                        .errors
                        .push_err(spanned_err!(lt, "unknown lifetime"));
                    LifetimeIndex::NONE
                }
            }
        }
        .piped(|lt| self.vars.add_referenced_env_lifetime(lt))
    }

    fn visit_type_macro_mut(&mut self, i: &mut syn::TypeMacro) {
        if !self.vars.allow_type_macros {
            push_type_macro_err(&mut self.vars.errors, i);
        }
    }
}

/////////////

impl<'a, 'b> FnVisitor<'a, 'b> {
    /// This function does these things:
    ///
    /// - Adds the lifetime as a referenced lifetime.
    ///
    /// - If `lt` is `Some('someident)` returns `Some('_)`.
    ///
    #[inline(never)]
    fn setup_lifetime(&mut self, lt: Option<&Ident>, span: Span) -> Option<&'a Ident> {
        let ctokens = self.refs.ctokens;
        let mut ret: Option<&'a Ident> = None;
        if lt == Some(&ctokens.static_) {
            LifetimeIndex::STATIC
        } else if lt.is_none() || lt == Some(&ctokens.underscore) {
            match self.param_ret.param_or_ret {
                ParamOrReturn::Param => self.new_bound_lifetime(span),
                ParamOrReturn::Return => match self.current.bound_lts_count {
                    0 => {
                        self.vars.errors.push_err(syn_err!(
                            span,
                            "attempted to use an elided lifetime  in the \
                                 return type when there are no lifetimes \
                                 used in any parameter",
                        ));
                        LifetimeIndex::NONE
                    }
                    1 => LifetimeIndex::Param(self.vars.fn_info.initial_bound_lifetime as _),
                    _ => {
                        self.vars.errors.push_err(syn_err!(
                            span,
                            "attempted to use an elided lifetime in the \
                                 return type when there are multiple lifetimes used \
                                 in parameters.",
                        ));
                        LifetimeIndex::NONE
                    }
                },
            }
        } else {
            let lt = lt.expect("BUG");
            let env_lts = self.vars.fn_info.env_lifetimes.iter();
            let fn_lts = self.current.named_bound_lts.iter();
            let found_lt = env_lts.chain(fn_lts).position(|ident| *ident == lt);
            match found_lt {
                Some(index) => {
                    if let Some(index) = index.checked_sub(self.current.first_bound_lt) {
                        self.current.bound_lt_spans[index].get_or_insert(span);
                    }
                    ret = Some(&ctokens.underscore);
                    LifetimeIndex::Param(index as _)
                }
                None => {
                    self.vars
                        .errors
                        .push_err(spanned_err!(lt, "unknown lifetime"));
                    LifetimeIndex::NONE
                }
            }
        }
        .piped(|li| {
            self.param_ret.lifetime_refs.push(li);
            self.lifetime_counts.increment(li);
        });
        ret
    }

    /// Adds a bound lifetime to the `extern "C" fn()` and returns an index to it
    fn new_bound_lifetime(&mut self, span: Span) -> LifetimeIndex {
        let index = self.vars.fn_info.initial_bound_lifetime + self.current.bound_lts_count;
        self.current.bound_lt_spans.push(Some(span));
        self.current.bound_lts_count += 1;
        LifetimeIndex::Param(index as _)
    }
}

impl<'a, 'b> VisitMut for FnVisitor<'a, 'b> {
    #[inline(never)]
    fn visit_type_bare_fn_mut(&mut self, func: &mut TypeBareFn) {
        self.vars.errors.push_err(syn_err!(
            self.param_ret.span,
            "\n\
             This library does not currently support nested function pointers.\n\
             To use the function pointer as a parameter define a wrapper type:\n\
             \t#[derive(StableAbi)]\n\
             \t#[repr(transparent)] \n\
             \tpub struct CallbackParam{{   \n\
             \t\tpub func:{func}\n\
             \t}}\n\
             \n\
             ",
            func = func.to_token_stream()
        ))
    }

    /// Visits references inside the function pointer type,
    /// uneliding their lifetime parameter,
    /// and pushing the lifetime to the list of lifetime indices.
    fn visit_type_reference_mut(&mut self, ref_: &mut TypeReference) {
        let _ctokens = self.refs.ctokens;
        let lt = ref_.lifetime.as_ref().map(|x| &x.ident);
        if let Some(ident) = self.setup_lifetime(lt, ref_.and_token.span()).cloned() {
            if let Some(lt) = &mut ref_.lifetime {
                lt.ident = ident
            }
        }

        // Visits the `Foo` type in a `&'a Foo`.
        visit_mut::visit_type_mut(self, &mut ref_.elem)
    }

    /// Visits a lifetime inside the function pointer type,
    /// and pushing the lifetime to the list of lifetime indices.
    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        if let Some(ident) = self.setup_lifetime(Some(&lt.ident), lt.apostrophe.span()) {
            lt.ident = ident.clone();
        }
    }

    fn visit_type_macro_mut(&mut self, i: &mut syn::TypeMacro) {
        if !self.vars.allow_type_macros {
            push_type_macro_err(&mut self.vars.errors, i);
        }
    }
}

/////////////

fn extract_fn_arg_name<'a>(
    _index: usize,
    arg: &mut syn::BareFnArg,
    arenas: &'a Arenas,
) -> Option<&'a Ident> {
    match arg.name.take() {
        Some((name, _)) => Some(arenas.alloc(name)),
        None => None,
    }
}

/////////////

impl<'a> Function<'a> {
    /// Turns lifetimes in the function parameters that aren't
    /// used in the return type or used only once into LifeimeIndex::ANONYMOUS,
    fn anonimize_lifetimes(
        &mut self,
        lifetime_counts: &LifetimeCounters,
        errors: &mut Result<(), syn::Error>,
    ) {
        let first_bound_lt = self.first_bound_lt;

        let mut current_lt = first_bound_lt;

        let asigned_lts = (0..self.bound_lts_count)
            .map(|i| {
                let lt_i: usize = first_bound_lt + i;

                if lifetime_counts.get(LifetimeIndex::Param(lt_i)) <= 1 {
                    LifetimeIndex::ANONYMOUS
                } else {
                    if current_lt == LifetimeIndex::MAX_LIFETIME_PARAM + 1 {
                        errors.push_err(syn_err!(
                            self.bound_lt_spans[i].unwrap_or(*self.func_span),
                            "Cannot have more than {} non-static lifetimes \
                             (except for lifetimes only used once inside \
                             function pointer types)",
                            LifetimeIndex::MAX_LIFETIME_PARAM + 1
                        ));
                    }

                    let ret = LifetimeIndex::Param(current_lt);
                    current_lt += 1;
                    ret
                }
            })
            .collect::<Vec<LifetimeIndex>>();

        for params in &mut self.params {
            for p_lt in &mut params.lifetime_refs {
                let param = match p_lt.to_param() {
                    Some(param) => (param).wrapping_sub(first_bound_lt),
                    None => continue,
                };

                if let Some(assigned) = asigned_lts.get(param) {
                    *p_lt = *assigned;
                }
            }
        }
    }
}

fn push_type_macro_err(res: &mut Result<(), syn::Error>, i: &syn::TypeMacro) {
    res.push_err(spanned_err!(
        i,
        "\
Cannot currently use type macros safely.

To enable use of type macros use the `#[sabi(unsafe_allow_type_macros)]` attribute.

The reason this is unsafe to enable them is because StableAbi cannot currently 
analize the lifetimes within macros,
which means that if any lifetime argument inside the macro invocation changes
it won't be checked by the runtime type checker.

"
    ));
}
