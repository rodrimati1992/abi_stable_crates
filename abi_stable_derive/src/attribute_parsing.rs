use as_derive_utils::parse_utils::ParseBufferExt;

use syn::{parse::ParseBuffer, Token};

///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

mod kw {
    syn::custom_keyword! {hidden}
}

// Searches for `hidden` within a `doc` attribute
pub(crate) fn contains_doc_hidden(input: &ParseBuffer<'_>) -> Result<bool, syn::Error> {
    let input = if input.peek(Token!(=)) {
        input.ignore_rest();
        return Ok(false);
    } else {
        input.parse_paren_buffer()?
    };

    let mut is_hidden = false;
    while !input.is_empty() {
        if input.check_parse(kw::hidden)? {
            is_hidden = true;
            if !input.is_empty() {
                let _ = input.parse::<Token!(,)>()?;
            }
            break;
        } else {
            input.skip_tt();
        }
    }
    Ok(is_hidden)
}
