use syn::{
    visit_mut::{
        VisitMut,
        visit_ident_mut,
    },
    spanned::Spanned,
    Ident,
};


use proc_macro2::Span;


#[derive(Debug,Clone,Copy)]
pub struct SetSpanVisitor{
    pub span:Span,
}

impl SetSpanVisitor{
    pub fn new(span:Span)->Self{
        Self{span}
    }
    pub fn span_of<T>(thing:&T)->Self
    where
        T:Spanned,
    {
        Self{
            span:thing.span()
        }
    }
}


impl VisitMut for SetSpanVisitor{
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        i.set_span(self.span);
    }
}