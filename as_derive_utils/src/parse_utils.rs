use crate::utils::SynErrorExt;

use syn::parse::{Parse, ParseBuffer, Peek};

#[doc(hidden)]
#[macro_export]
macro_rules! ret_err_on_peek {
    ($input:ident, $peeked:expr, $expected:expr, $found_msg:expr $(,)*) => {
        if $input.peek($peeked) {
            return Err($input.error(concat!("expected ", $expected, ", found ", $found_msg)));
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! ret_err_on {
    ($cond:expr, $span:expr, $expected:expr, $found_msg:expr $(,)*) => {
        if $cond {
            return Err(syn::Error::new(
                $span,
                concat!("expected ", $expected, ", found ", $found_msg),
            ));
        }
    };
}

pub use crate::{ret_err_on, ret_err_on_peek};

fn parse_with_prefix_err<T>(input: &ParseBuffer<'_>, prefix: &str) -> Result<T, syn::Error>
where
    T: Parse,
{
    input
        .parse::<T>()
        .map_err(|e| e.prepend_msg(format!("{}: ", prefix)))
}

pub trait ParseBufferExt {
    fn as_pb(&self) -> &ParseBuffer<'_>;

    fn peek_parse<F, X, P>(&self, f: F) -> Result<Option<P>, syn::Error>
    where
        F: FnOnce(X) -> P + Peek,
        P: Parse,
    {
        let this = self.as_pb();
        if this.peek(f) {
            this.parse::<P>().map(Some)
        } else {
            Ok(None)
        }
    }

    /// Alternate method for parsing a type, with a better (?) error message.
    fn parse_type(&self) -> Result<syn::Type, syn::Error> {
        let input = self.as_pb();

        if input.peek(syn::Lit) {
            Err(input.error("expected type, found literal"))
        } else {
            parse_with_prefix_err(input, "while parsing type")
        }
    }

    /// Alternate method for parsing an expression, with a better (?) error message.
    fn parse_expr(&self) -> Result<syn::Expr, syn::Error> {
        parse_with_prefix_err(self.as_pb(), "while parsing expression")
    }

    /// skips a token tree.
    fn skip_tt(&self) {
        let _ = self.as_pb().parse::<proc_macro2::TokenTree>();
    }

    /// Ignores the rest of the parse input
    fn ignore_rest(&self) {
        let _ = self.as_pb().parse::<proc_macro2::TokenStream>();
    }

    /// Checks that a token is parsable, advancing the parse buffer if it is.
    ///
    /// This returns:
    /// - Ok(true): if the token was the passed in one, advancing the parser.
    /// - Ok(false): if the token was not passed in one, keeping the token unparsed.
    /// - Err: if there was an error parsing the token
    fn check_parse<F, X, P>(&self, f: F) -> Result<bool, syn::Error>
    where
        F: FnOnce(X) -> P + Peek,
        P: Parse,
    {
        let this = self.as_pb();
        if this.peek(f) {
            match this.parse::<P>() {
                Ok(_) => Ok(true),
                Err(e) => Err(e),
            }
        } else {
            Ok(false)
        }
    }

    fn parse_paren_buffer(&self) -> Result<ParseBuffer, syn::Error> {
        let content;
        let _ = syn::parenthesized!(content in self.as_pb());
        Ok(content)
    }

    fn parse_paren_as<T>(&self) -> Result<T, syn::Error>
    where
        T: Parse,
    {
        self.as_pb().parse_paren_with(|x| x.parse::<T>())
    }

    fn parse_paren_with<T, F>(&self, f: F) -> Result<T, syn::Error>
    where
        F: FnOnce(&ParseBuffer<'_>) -> Result<T, syn::Error>,
    {
        let content;
        let _ = syn::parenthesized!(content in self.as_pb());
        f(&content)
    }

    fn parse_int<N>(&self) -> Result<N, syn::Error>
    where
        N: std::str::FromStr,
        N::Err: std::fmt::Display,
    {
        self.as_pb().parse::<syn::LitInt>()?.base10_parse::<N>()
    }

    fn for_each_separated<F, G, P>(&self, _sep: G, mut func: F) -> Result<(), syn::Error>
    where
        F: FnMut(&ParseBuffer<'_>) -> Result<(), syn::Error>,
        G: Fn(proc_macro2::Span) -> P + Copy,
        P: Parse,
    {
        let this = self.as_pb();

        if this.is_empty() {
            return Ok(());
        }

        loop {
            func(this)?;

            if !this.is_empty() {
                let _ = this.parse::<P>()?;
            }
            if this.is_empty() {
                break Ok(());
            }
        }
    }
}

impl ParseBufferExt for ParseBuffer<'_> {
    #[inline(always)]
    fn as_pb(&self) -> &ParseBuffer<'_> {
        self
    }
}
