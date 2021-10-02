use syn::{spanned::Spanned, visit_mut::VisitMut, Ident};

use proc_macro2::Span;

/// Used to set the span of all identifier of the thing it's visiting.
#[derive(Debug, Clone, Copy)]
pub struct SetSpanVisitor {
    pub span: Span,
}

impl SetSpanVisitor {
    pub fn new(span: Span) -> Self {
        Self { span }
    }
    #[allow(dead_code)]
    pub fn span_of<T>(thing: &T) -> Self
    where
        T: Spanned,
    {
        Self { span: thing.span() }
    }
}

impl VisitMut for SetSpanVisitor {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        i.set_span(self.span);
    }
}
