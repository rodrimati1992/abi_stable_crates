use syn::{
    visit_mut::{self, VisitMut},
    Lifetime, Type, TypeReference,
};

/// Used to unelide the lifetimes in the `&self` parameter of methods.
pub(crate) struct LifetimeUnelider<'a, 'b> {
    self_lifetime: &'b mut Option<&'a syn::Lifetime>,
    contains_self_borrow: bool,
    pub(crate) additional_lifetime_def: Option<&'a syn::LifetimeDef>,
}

pub(crate) struct TypeProperties<'a> {
    pub(crate) additional_lifetime_def: Option<&'a syn::LifetimeDef>,
    pub(crate) found_borrow_kind: Option<BorrowKind>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BorrowKind {
    Reference,
    MutReference,
    Other,
}

impl<'a, 'b> LifetimeUnelider<'a, 'b> {
    pub(crate) fn new(self_lifetime: &'b mut Option<&'a syn::Lifetime>) -> Self {
        Self {
            self_lifetime,
            contains_self_borrow: false,
            additional_lifetime_def: None,
        }
    }

    /// Unelide the lifetimes the `&self` parameter of methods.
    pub(crate) fn visit_type(mut self, ty: &mut Type) -> TypeProperties<'a> {
        self.contains_self_borrow = false;

        visit_mut::visit_type_mut(&mut self, ty);

        let found_borrow_kind = if self.contains_self_borrow {
            let bk = match &*ty {
                Type::Reference(tr) => {
                    if tr.mutability.is_some() {
                        BorrowKind::MutReference
                    } else {
                        BorrowKind::Reference
                    }
                }
                _ => BorrowKind::Other,
            };

            Some(bk)
        } else {
            None
        };

        TypeProperties {
            additional_lifetime_def: self.additional_lifetime_def,
            found_borrow_kind,
        }
    }
}

impl<'a, 'b> LifetimeUnelider<'a, 'b> {
    fn setup_lifetime(&mut self) -> Lifetime {
        let additional_lifetime_def = &mut self.additional_lifetime_def;
        let x = self.self_lifetime.get_or_insert_with(|| {
            let ret = syn::parse_str::<syn::LifetimeDef>("'_self").unwrap();
            let ret: &'a syn::LifetimeDef = Box::leak(Box::new(ret));
            *additional_lifetime_def = Some(ret);
            &ret.lifetime
        });
        (*x).clone()
    }
}
impl<'a, 'b> VisitMut for LifetimeUnelider<'a, 'b> {
    fn visit_type_reference_mut(&mut self, ref_: &mut TypeReference) {
        if is_self_borrow(self.self_lifetime, ref_) {
            self.contains_self_borrow = true;
        }

        let is_elided = ref_.lifetime.as_ref().map_or(true, |x| x.ident == "_");

        if is_elided {
            ref_.lifetime = Some(self.setup_lifetime());
        }

        visit_mut::visit_type_mut(self, &mut ref_.elem)
    }

    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        if is_self_lifetime(self.self_lifetime, lt) {
            self.contains_self_borrow = true;
        }

        if lt.ident == "_" {
            *lt = self.setup_lifetime();
        }
    }
}

fn is_self_lifetime(self_lifetime: &Option<&Lifetime>, lt: &Lifetime) -> bool {
    match self_lifetime {
        Some(sl) if sl.ident == lt.ident => true,
        _ => lt.ident == "_",
    }
}

fn is_self_borrow(self_lifetime: &Option<&Lifetime>, tr: &TypeReference) -> bool {
    match (self_lifetime, &tr.lifetime) {
        (Some(sl), Some(lt)) if sl.ident == lt.ident => true,
        (_, Some(lt)) => lt.ident == "_",
        (_, None) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_self_borrow_kind<'a, 'b>(
        mut self_lifetime: Option<&'a syn::Lifetime>,
        mut ty: Type,
    ) -> Option<BorrowKind> {
        let mut this = LifetimeUnelider::new(&mut self_lifetime);
        let ret = this.visit_type(&mut ty.clone());
        ret.found_borrow_kind
    }

    fn assert_elided(self_lifetime: &Lifetime, ty: Type, expected: BorrowKind) {
        assert_eq!(
            get_self_borrow_kind(Some(self_lifetime), ty.clone()),
            Some(expected.clone())
        );
        assert_eq!(get_self_borrow_kind(None, ty.clone()), Some(expected));
    }

    fn assert_unelided(self_lifetime: &Lifetime, ty: Type, expected: BorrowKind) {
        assert_eq!(
            get_self_borrow_kind(Some(self_lifetime), ty.clone()),
            Some(expected.clone())
        );
        assert_eq!(get_self_borrow_kind(None, ty.clone()), None);
    }

    fn parse_ty(s: &str) -> syn::Type {
        syn::parse_str(s).unwrap()
    }

    #[test]
    fn borrow_self_detection() {
        let lifetime_a = &syn::parse_str::<Lifetime>("'a").unwrap();

        // Implicitly borrowing from self
        {
            assert_elided(lifetime_a, parse_ty("&()"), BorrowKind::Reference);
            assert_elided(lifetime_a, parse_ty("&mut ()"), BorrowKind::MutReference);
            assert_elided(lifetime_a, parse_ty("Option<&()>"), BorrowKind::Other);
            assert_elided(lifetime_a, parse_ty("Option<&mut ()>"), BorrowKind::Other);
            assert_elided(lifetime_a, parse_ty("Foo<'_>"), BorrowKind::Other);

            assert_elided(lifetime_a, parse_ty("&'_ &'b ()"), BorrowKind::Reference);
            assert_elided(
                lifetime_a,
                parse_ty("&'_ mut &'b ()"),
                BorrowKind::MutReference,
            );
            assert_elided(lifetime_a, parse_ty("&'b &'_ ()"), BorrowKind::Reference);
            assert_elided(
                lifetime_a,
                parse_ty("&'b mut &'_ ()"),
                BorrowKind::MutReference,
            );

            assert_elided(
                lifetime_a,
                parse_ty("Option<&'_ &'b ()>"),
                BorrowKind::Other,
            );
            assert_elided(
                lifetime_a,
                parse_ty("Option<&'_ mut &'b ()>"),
                BorrowKind::Other,
            );
            assert_elided(
                lifetime_a,
                parse_ty("Option<&'b &'_ ()>"),
                BorrowKind::Other,
            );
            assert_elided(
                lifetime_a,
                parse_ty("Option<&'b mut &'_ ()>"),
                BorrowKind::Other,
            );
        }

        // Explicitly borrowing from self
        {
            assert_unelided(lifetime_a, parse_ty("&'a ()"), BorrowKind::Reference);
            assert_unelided(lifetime_a, parse_ty("&'a mut ()"), BorrowKind::MutReference);
            assert_unelided(lifetime_a, parse_ty("Option<&'a ()>"), BorrowKind::Other);
            assert_unelided(
                lifetime_a,
                parse_ty("Option<&'a mut ()>"),
                BorrowKind::Other,
            );
            assert_unelided(lifetime_a, parse_ty("Foo<'a>"), BorrowKind::Other);

            assert_unelided(lifetime_a, parse_ty("&'a &'b ()"), BorrowKind::Reference);
            assert_unelided(
                lifetime_a,
                parse_ty("&'a mut &'b ()"),
                BorrowKind::MutReference,
            );
            assert_unelided(lifetime_a, parse_ty("&'b &'a ()"), BorrowKind::Reference);
            assert_unelided(
                lifetime_a,
                parse_ty("&'b mut &'a ()"),
                BorrowKind::MutReference,
            );

            assert_unelided(
                lifetime_a,
                parse_ty("Option<&'a &'b ()>"),
                BorrowKind::Other,
            );
            assert_unelided(
                lifetime_a,
                parse_ty("Option<&'a mut &'b ()>"),
                BorrowKind::Other,
            );
            assert_unelided(
                lifetime_a,
                parse_ty("Option<&'b &'a ()>"),
                BorrowKind::Other,
            );
            assert_unelided(
                lifetime_a,
                parse_ty("Option<&'b mut &'a ()>"),
                BorrowKind::Other,
            );
        }

        {
            let gsbk = get_self_borrow_kind;
            let lt_a = Some(lifetime_a);
            assert_eq!(gsbk(lt_a, parse_ty("&'b ()")), None);
            assert_eq!(gsbk(lt_a, parse_ty("&'b mut ()")), None);
            assert_eq!(gsbk(lt_a, parse_ty("Option<&'b ()>")), None);
            assert_eq!(gsbk(lt_a, parse_ty("Option<&'b mut ()>")), None);
            assert_eq!(gsbk(lt_a, parse_ty("Foo<'b>")), None);
        }
    }
}
