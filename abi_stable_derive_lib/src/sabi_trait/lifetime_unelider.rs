use super::*;

use syn::{
    visit_mut::{self, VisitMut},
    Lifetime, TypeReference, Type,
};


/// Used to unelide the lifetimes in the `&self` parameter of methods.
pub(crate) struct LifetimeUnelider<'a,'b>{
    ctokens: &'a CommonTokens,
    self_lifetime:&'b mut Option<&'a syn::Lifetime>,
    pub(crate) additional_lifetime_def:Option<&'a syn::LifetimeDef>
}

impl<'a,'b> LifetimeUnelider<'a,'b>{
    pub(crate) fn new(
        ctokens: &'a CommonTokens,
        self_lifetime:&'b mut Option<&'a syn::Lifetime>,
    )->Self{
        Self{
            ctokens,
            self_lifetime,
            additional_lifetime_def:None,
        }
    }

    /// Unelide the lifetimes the `&self` parameter of methods.
    pub(crate) fn visit_type(mut self,ty: &mut Type)->Option<&'a syn::LifetimeDef>{
        visit_mut::visit_type_mut(&mut self,ty);
        self.additional_lifetime_def
    }
}

impl<'a,'b> LifetimeUnelider<'a,'b> {
    fn setup_lifetime(&mut self)->Lifetime{
        let ct=self.ctokens;
        let additional_lifetime_def=&mut self.additional_lifetime_def;
        let x=self.self_lifetime.get_or_insert_with(||{
            *additional_lifetime_def=Some(&ct.uself_lt_def);
            &ct.uself_lifetime
        });
        (*x).clone()
    }
}
impl<'a,'b> VisitMut for LifetimeUnelider<'a,'b> {
    fn visit_type_reference_mut(&mut self, ref_: &mut TypeReference) {
        let is_elided=ref_.lifetime.as_ref().map_or(true,|x| *x==self.ctokens.under_lifetime );

        if is_elided{
            ref_.lifetime=Some(self.setup_lifetime());
        }
    }

    fn visit_lifetime_mut(&mut self, lt: &mut Lifetime) {
        if *lt==self.ctokens.under_lifetime{
            *lt=self.setup_lifetime();
        }
    }
}