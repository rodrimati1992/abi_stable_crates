/*!
Contains visitor type for 
extracting function pointers and the referenced lifetimes of the fields of a type declaration.
*/

use std::{
    collections::HashSet,
    mem,
};


use core_extensions::prelude::*;

use syn::{
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{self, VisitMut},
    Generics, Ident, Lifetime, Type, TypeBareFn, TypeReference,
};

use proc_macro2::Span;

use quote::ToTokens;



use crate::{
    common_tokens::FnPointerTokens,
    lifetimes::LifetimeIndex,
    ignored_wrapper::Ignored,
    utils::SynResultExt,
};
use crate::*;

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
    /// The named lifetimes for this function pointer type,
    /// the ones declared within `for<'a,'b,'c>`.
    pub(crate) named_bound_lts: Vec<&'a Ident>,
    /// A set version of the `named_bound_lts` field.
    pub(crate) named_bound_lt_set: Ignored<HashSet<&'a Ident>>,
    /// The amount of named lifetimes declared by the function pointer.
    pub(crate) named_bound_lts_count:usize,

    pub(crate) is_unsafe: bool,

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
    pub(crate) name: Option<&'a str>,
    /// The lifetimes this type references (including static).
    pub(crate) lifetime_refs: Vec<LifetimeIndex>,
    /// The type of the parameter/return type.
    pub(crate) ty: &'a Type,
    /// Whether this is a parameter or a return type.
    pub(crate) param_or_ret: ParamOrReturn,
}

impl<'a> FnParamRet<'a>{
    /// Constructs an `FnParamRet` for a `()` return type.
    pub fn unit_ret(arenas:&'a Arenas)->Self{
        let unit=syn::TypeTuple{
            paren_token:Default::default(),
            elems:Punctuated::default(),
        };
        FnParamRet{
            name:None,
            lifetime_refs:Vec::new(),
            ty:arenas.alloc( syn::Type::from(unit)  ),
            param_or_ret:ParamOrReturn::Return,
        }
    }
}

/// The information returned from visiting a field.
pub(crate) struct VisitFieldRet<'a> {
    /// The lifetimes that the field references.
    pub(crate) referenced_lifetimes: Vec<LifetimeIndex>,
    /// The function pointer types in the field.
    pub(crate) functions:Vec<Function<'a>>,
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
                referenced_lifetimes: Vec::default(),
                fn_info: FnInfo {
                    parent_generics: &generics,
                    env_lifetimes: generics.lifetimes().map(|lt| &lt.lifetime.ident).collect(),
                    initial_bound_lifetime: generics.lifetimes().count(),
                    functions: Vec::new(),
                },
                errors:Ok(()),
            },
        }
    }

    /// Gets the arena this references.
    pub fn arenas(&self)->&'a Arenas{
        self.refs.arenas
    }
    /// Gets the CommonTokens this references.
    pub fn ctokens(&self)->&'a FnPointerTokens{
        self.refs.ctokens
    }
    /// Gets the generic parameters this references.
    pub fn env_generics(&self)->&'a Generics{
        self.refs.env_generics
    }

    /// Visit a field type,
    /// returning the function pointer types and referenced lifetimes.
    pub fn visit_field(&mut self,ty: &mut Type) -> VisitFieldRet<'a> {
        self.visit_type_mut(ty);

        VisitFieldRet {
            referenced_lifetimes: mem::replace(&mut self.vars.referenced_lifetimes, Vec::new()),
            functions:mem::replace(&mut self.vars.fn_info.functions, Vec::new()),
        }
    }

    pub fn get_errors(&mut self)->Result<(),syn::Error>{
        mem::replace(&mut self.vars.errors,Ok(()))
    }

    pub fn into_fn_info(self)->FnInfo<'a>{
        self.vars.fn_info
    }
}

/////////////

/// Whether this is a parameter or a return type/value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate)enum ParamOrReturn {
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
    /// What lifetimes in env_lifetimes are referenced in the type being visited.
    /// For TLField.
    referenced_lifetimes: Vec<LifetimeIndex>,
    fn_info: FnInfo<'a>,
    errors: Result<(),syn::Error>,
}

/// Used to visit a function pointer type.
struct FnVisitor<'a, 'b> {
    refs: ImmutableRefs<'a>,
    vars: &'b mut Vars<'a>,

    /// The current function pointer type that is being visited,
    current: Function<'a>,
    /// The lifetime indices inside a parameter/return type that is currently being visited.
    param_ret: FnParamRetLifetimes,
}

/// The lifetime indices inside a parameter/return type.
struct FnParamRetLifetimes {
    span:Span,//TODO
    /// The lifetimes this type references (including static).
    lifetime_refs: Vec<LifetimeIndex>,
    /// Whether this is a parameter or return type.
    param_or_ret: ParamOrReturn,
}

/////////////

impl FnParamRetLifetimes {
    fn new(param_or_ret: ParamOrReturn, span: Option<Span>) -> Self {
        Self {
            span:span.unwrap_or_else(Span::call_site),
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
        let is_env_lt = match ind {
            LifetimeIndex::Static => true,
            LifetimeIndex::Param { index } => (index as usize) < self.fn_info.env_lifetimes.len(),
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

        let is_unsafe=func.unsafety.is_some();

        let abi = func
            .abi
            .as_ref()
            .map(|x| x.name.as_ref().unwrap_or(&ctokens.c_abi_lit));

        if abi != Some(&ctokens.c_abi_lit) {
            match abi {
                Some(abi) => self.vars.errors.push_err(spanned_err!(
                    abi,
                    "Abi not supported for function pointers", 
                )),
                None => self.vars.errors.push_err(spanned_err!(
                    func,
                    "The default abi is not supported for function pointers.",
                )),
            }
        }

        let named_bound_lts: Vec<&'a Ident> = func
            .lifetimes
            .take() // Option<BoundLifetimes>
            .into_iter()
            .flat_map(|lt| lt.lifetimes)
            .map(|lt| arenas.alloc(lt.lifetime.ident))
            .collect::<Vec<&'a Ident>>();

        let named_bound_lt_set=named_bound_lts.iter().cloned().collect();


        let mut current_function = FnVisitor {
            refs: self.refs,
            vars: &mut self.vars,
            current: Function {
                named_bound_lts_count:named_bound_lts.len(),
                named_bound_lts,
                named_bound_lt_set:Ignored::new(named_bound_lt_set),
                is_unsafe,
                params: Vec::new(),
                returns: None,
            },
            param_ret: FnParamRetLifetimes::new(ParamOrReturn::Param,None),
        };

        // Visits a parameter or return type within a function pointer type.
        fn visit_param_ret<'a, 'b>(
            this: &mut FnVisitor<'a, 'b>,
            name:Option<&'a str>,
            ty: &'a mut Type,
            param_or_ret: ParamOrReturn,
        ) {
            let ty_span=Some(ty.span());

            this.param_ret = FnParamRetLifetimes::new(param_or_ret,ty_span);

            this.visit_type_mut(ty);

            let param_ret = mem::replace(
                &mut this.param_ret, 
                FnParamRetLifetimes::new(param_or_ret,ty_span)
            );

            let param_ret = FnParamRet {
                name,
                lifetime_refs: param_ret.lifetime_refs,
                ty: ty,
                param_or_ret: param_ret.param_or_ret,
            };

            match param_or_ret {
                ParamOrReturn::Param => this.current.params.push(param_ret),
                ParamOrReturn::Return => this.current.returns = Some(param_ret),
            }
        }

        for (i,mut param) in 
            mem::replace(&mut func.inputs, Punctuated::new())
                .into_iter()
                .enumerate() 
        {
            let arg_name=extract_fn_arg_name(i,&mut param,arenas);
            let ty = arenas.alloc_mut(param.ty);
            visit_param_ret(&mut current_function,arg_name, ty, ParamOrReturn::Param);
        }

        match mem::replace(&mut func.output, syn::ReturnType::Default) {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ty) => Some(arenas.alloc_mut(*ty)),
        }
        .map(|ty| visit_param_ret(&mut current_function,None, ty, ParamOrReturn::Return));

        let current=current_function.current;
        self.vars.fn_info.functions.push(current);
    }

    /// Visits a lifetime within a field type,
    /// pushing it to the list of referenced lifetimes.
    #[inline(never)]
    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        let ctokens = self.refs.ctokens;
        let lt = &lt.ident;
        if *lt == ctokens.static_ {
            LifetimeIndex::Static
        } else {
            let env_lifetimes = self.vars.fn_info.env_lifetimes.iter();
            let found_lt = env_lifetimes.enumerate().position(|(_, lt_ident)| *lt_ident == lt);
            match found_lt {
                Some(index) => LifetimeIndex::Param { index:index as _ },
                None => {
                    self.vars.errors.push_err(spanned_err!(lt,"unknown lifetime"));
                    LifetimeIndex::Static
                }
            }
        }
        .piped(|lt| self.vars.add_referenced_env_lifetime(lt))
    }
}

/////////////

impl<'a, 'b> FnVisitor<'a, 'b> {
    /**
This function does these things:

- Adds the lifetime as a referenced lifetime.

- If `lt` is `Some('someident)` returns `Some('_)`.

    */
    #[inline(never)]
    fn setup_lifetime(&mut self, lt: Option<&Ident>) -> Option<&'a Ident> {
        let ctokens = self.refs.ctokens;
        let mut ret: Option<&'a Ident> = None;
        if lt == Some(&ctokens.static_) {
            LifetimeIndex::Static
        } else if lt==None || lt==Some(&ctokens.underscore) {
            match self.param_ret.param_or_ret {
                ParamOrReturn::Param => {
                    self.new_bound_lifetime()
                },
                ParamOrReturn::Return => 
                    match self.current.named_bound_lts_count {
                        0 =>{
                            self.vars.errors.push_err(syn_err!(
                                self.param_ret.span,
                                "attempted to use an elided lifetime  in the \
                                 return type when there are no lifetimes \
                                 used in any parameter",
                            ));
                            LifetimeIndex::Static
                        } 
                        1=> {
                            LifetimeIndex::Param {
                                index: self.vars.fn_info.initial_bound_lifetime as _,
                            }
                        }
                        _ =>{
                            self.vars.errors.push_err(syn_err!(
                                self.param_ret.span,
                                "attempted to use an elided lifetime in the \
                                 return type when there are multiple lifetimes used \
                                 in parameters.",
                            ));
                            LifetimeIndex::Static
                        }
                    },
            }
        } else {
            let lt=lt.expect("BUG");
            let env_lts = self.vars.fn_info.env_lifetimes.iter();
            let fn_lts = self.current.named_bound_lts.iter();
            let found_lt = env_lts.chain(fn_lts).position(|ident| *ident == lt);
            match found_lt {
                Some(index) => {
                    ret = Some(&ctokens.underscore);
                    LifetimeIndex::Param { index:index as _ }
                }
                None => {
                    self.vars.errors.push_err(spanned_err!(lt,"unknown lifetime"));
                    LifetimeIndex::Static
                },
            }
        }
        .piped(|li| {
            self.param_ret.lifetime_refs.push(li);
            self.vars.add_referenced_env_lifetime(li);
        });
        ret
    }
    
    /// Adds a bound lifetime to the `extern fn()` and returns an index to it
    fn new_bound_lifetime(&mut self) -> LifetimeIndex {
        let index = self.vars.fn_info.initial_bound_lifetime+self.current.named_bound_lts_count;
        self.current.named_bound_lts_count+=1;
        LifetimeIndex::Param { index:index as _ }
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
             func=func.to_token_stream()
        ))
    }

    /// Visits references inside the function pointer type,
    /// uneliding their lifetime parameter,
    /// and pushing the lifetime to the list of lifetime indices.
    fn visit_type_reference_mut(&mut self, ref_: &mut TypeReference) {
        let _ctokens = self.refs.ctokens;
        let lt = ref_.lifetime.as_ref().map(|x| &x.ident);
        if let Some(ident) = self.setup_lifetime(lt).cloned() {
            if let Some(lt)=&mut ref_.lifetime {
                lt.ident = ident
            }
        }

        // Visits the `Foo` type in a `&'a Foo`.
        visit_mut::visit_type_mut(self, &mut ref_.elem)
    }

    /// Visits a lifetime inside the function pointer type,
    /// and pushing the lifetime to the list of lifetime indices.
    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        if let Some(ident) = self.setup_lifetime(Some(&lt.ident)) {
            lt.ident = ident.clone();
        }
    }
}

/////////////

fn extract_fn_arg_name<'a>(
    _index:usize,
    arg:&mut syn::BareFnArg,
    arenas: &'a Arenas,
)->Option<&'a str>{
    match arg.name.take() {
        Some((name,_))=>Some(arenas.alloc(name.to_string())),
        None=>None,
    }
}