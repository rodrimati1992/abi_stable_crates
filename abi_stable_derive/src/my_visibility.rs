//! Types for conveniently representing visibility.

use proc_macro2::TokenStream;
use quote::ToTokens;
#[allow(unused_imports)]
use syn::{
    self,
    token::{Colon2, Crate, In, Paren, Pub, Super},
    Path, Visibility,
};

use core_extensions::SelfOps;

use std::cmp::{Ordering, PartialOrd};

/// A visibility in a nested module.
#[derive(Copy, Clone, Debug)]
pub(crate) struct RelativeVis<'a> {
    visibility_kind: VisibilityKind<'a>,
    nesting: u8,
}

/// A visibility.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VisibilityKind<'a> {
    Private,
    /// 'super' is 1,'super::super is 2,etc.
    Super {
        nth_supermod: usize,
    },
    Absolute(&'a Path),
    Crate,
    Public,
}

impl<'a> VisibilityKind<'a> {
    pub fn new(vis: &'a Visibility) -> Self {
        match vis {
            Visibility::Public { .. } => VisibilityKind::Public,
            Visibility::Crate { .. } => VisibilityKind::Crate,
            Visibility::Inherited { .. } => VisibilityKind::Private,
            Visibility::Restricted(restricted) => {
                let path = &restricted.path;
                let is_global = restricted.path.leading_colon.is_some();
                let path_seg_0 = path.segments.first();
                let is_crate = path_seg_0.map_or(false, |x| x.ident == "crate");
                if is_global || is_crate {
                    if is_crate && path.segments.len() == 1 {
                        VisibilityKind::Crate
                    } else {
                        VisibilityKind::Absolute(path)
                    }
                } else if path_seg_0.map_or(false, |x| x.ident == "self") {
                    assert!(
                        path.segments.len() == 1,
                        "paths in pub(...) that start with 'self' \
                         must not be followed by anything:\n{:?}.\n",
                        path
                    );

                    VisibilityKind::Private
                } else if path_seg_0.map_or(false, |x| x.ident == "super") {
                    assert!(
                        path.segments.iter().all(|segment| segment.ident == "super"),
                        "paths in pub(...) that start with 'super' \
                         must only be followed by 'super':\n{:?}.\n",
                        path
                    );

                    VisibilityKind::Super {
                        nth_supermod: path.segments.len(),
                    }
                } else {
                    VisibilityKind::Absolute(path)
                }
            }
        }
    }

    /// Returns a type which outputs the visibility for items in \[sub-\]modules.
    ///
    /// nesting==0 means the module deriving this trait
    ///
    /// nesting==1 means the module below that.
    pub(crate) fn submodule_level(self, nesting: u8) -> RelativeVis<'a> {
        RelativeVis {
            visibility_kind: self,
            nesting,
        }
    }
}

impl<'a> ToTokens for VisibilityKind<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.submodule_level(0).to_tokens(tokens)
    }
}
impl<'a> ToTokens for RelativeVis<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.visibility_kind == VisibilityKind::Private && self.nesting == 0 {
            return;
        }

        match self.visibility_kind {
            VisibilityKind::Private | VisibilityKind::Super { .. } => {
                let supermod = match self.visibility_kind {
                    VisibilityKind::Private => 0,
                    VisibilityKind::Super { nth_supermod } => nth_supermod,
                    _ => unreachable!(),
                };

                let nesting = supermod + self.nesting as usize;

                Pub::default().to_tokens(tokens);
                Paren::default().surround(tokens, |tokens| {
                    In::default().to_tokens(tokens);

                    let mut iter = (0..nesting).peekable();
                    while iter.next().is_some() {
                        Super::default().to_tokens(tokens);
                        if iter.peek().is_some() {
                            Colon2::default().to_tokens(tokens);
                        }
                    }
                });
            }
            VisibilityKind::Absolute(path) => {
                Pub::default().to_tokens(tokens);
                Paren::default().surround(tokens, |tokens| {
                    In::default().to_tokens(tokens);
                    path.to_tokens(tokens);
                });
            }
            VisibilityKind::Crate => {
                Pub::default().to_tokens(tokens);
                Paren::default().surround(tokens, |tokens| {
                    Crate::default().to_tokens(tokens);
                });
            }
            VisibilityKind::Public => {
                Pub::default().to_tokens(tokens);
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Eq, PartialEq, PartialOrd, Ord)]
enum VKDiscr {
    Private,
    Super,
    Absolute,
    Crate,
    Public,
}

impl<'a> VisibilityKind<'a> {
    fn to_discriminant(self) -> VKDiscr {
        match self {
            VisibilityKind::Private { .. } => VKDiscr::Private,
            VisibilityKind::Super { .. } => VKDiscr::Super,
            VisibilityKind::Absolute { .. } => VKDiscr::Absolute,
            VisibilityKind::Crate { .. } => VKDiscr::Crate,
            VisibilityKind::Public { .. } => VKDiscr::Public,
        }
    }
}

impl<'a> PartialOrd for VisibilityKind<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use self::VisibilityKind as VK;

        match self.to_discriminant().cmp(&other.to_discriminant()) {
            expr @ Ordering::Less | expr @ Ordering::Greater => return Some(expr),
            _ => {}
        }

        match (self, other) {
            (&VK::Super { nth_supermod: nth0 }, &VK::Super { nth_supermod: nth1 }) => {
                nth0.partial_cmp(&nth1)
            }
            (&VK::Absolute(path0), &VK::Absolute(path1)) => {
                if path0
                    .segments
                    .iter()
                    .zip(&path1.segments)
                    .all(|(l, r)| l.ident == r.ident)
                {
                    path0
                        .segments
                        .len()
                        .cmp(&path1.segments.len())
                        .reverse()
                        .piped(Some)
                } else {
                    None
                }
            }
            _ => Some(Ordering::Equal),
        }
    }
}

// #[cfg(test)]
#[cfg(all(test, feature = "passed_tests"))]
mod tests {
    use super::*;

    #[test]
    fn test_ordering() {
        macro_rules! new_visibility {
            (
                $ident:ident=$string:expr
            ) => {
                let $ident: Visibility = syn::parse_str($string).expect($string);
                let $ident = VisibilityKind::new(&$ident).kind;
            };
        }

        new_visibility! {vis_self="pub(self)"}
        new_visibility! {vis_self_b=""}

        new_visibility! {vis_super="pub(super)"}
        new_visibility! {vis_super_1="pub(in super::super)"}
        new_visibility! {vis_super_2="pub(in super::super::super)"}

        new_visibility! {vis_mod1_mod2="pub(in  ::mod1::mod2)"}
        new_visibility! {vis_mod1_mod2_mod4="pub(in  ::mod1::mod2::mod4)"}
        new_visibility! {vis_mod1_mod3="pub(in ::mod1::mod3)"}
        new_visibility! {vis_mod1="pub(in ::mod1)"}

        new_visibility! {vis_crate="crate"}
        new_visibility! {vis_crate_1="pub(crate)"}

        new_visibility! {vis_pub="pub"}

        assert_eq!(vis_self, vis_self_b);
        assert_eq!(vis_crate, vis_crate_1);

        assert_eq!(vis_self.partial_cmp(&vis_super), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_super_1), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_super_2), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_mod1), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_mod1_mod2), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_crate), Some(Ordering::Less));
        assert_eq!(vis_self.partial_cmp(&vis_pub), Some(Ordering::Less));

        assert_eq!(vis_super.partial_cmp(&vis_super_1), Some(Ordering::Less));
        assert_eq!(vis_super.partial_cmp(&vis_super_2), Some(Ordering::Less));
        assert_eq!(vis_super_1.partial_cmp(&vis_super_2), Some(Ordering::Less));
        assert_eq!(vis_super_2.partial_cmp(&vis_mod1), Some(Ordering::Less));
        assert_eq!(
            vis_super_2.partial_cmp(&vis_mod1_mod2),
            Some(Ordering::Less)
        );
        assert_eq!(vis_super_2.partial_cmp(&vis_crate), Some(Ordering::Less));
        assert_eq!(vis_super_2.partial_cmp(&vis_pub), Some(Ordering::Less));

        assert_eq!(
            vis_mod1_mod2_mod4.partial_cmp(&vis_mod1_mod2),
            Some(Ordering::Less)
        );
        assert_eq!(
            vis_mod1_mod2.partial_cmp(&vis_mod1_mod2_mod4),
            Some(Ordering::Greater)
        );

        assert_eq!(vis_mod1_mod2.partial_cmp(&vis_mod1), Some(Ordering::Less));
        assert_eq!(vis_mod1_mod3.partial_cmp(&vis_mod1), Some(Ordering::Less));
        assert_eq!(vis_mod1_mod3.partial_cmp(&vis_mod1_mod2), None);

        assert_eq!(vis_mod1.partial_cmp(&vis_crate), Some(Ordering::Less));
        assert_eq!(vis_mod1.partial_cmp(&vis_pub), Some(Ordering::Less));

        assert_eq!(vis_crate.partial_cmp(&vis_pub), Some(Ordering::Less));
    }
}
