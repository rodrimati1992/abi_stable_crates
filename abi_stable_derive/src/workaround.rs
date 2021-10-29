//! Wrote this as a workaround for
//! `<TokenStream as Display>::fmt` not working correctly for some reason.

use std::{fmt::Write, mem};

use proc_macro2::{Delimiter, Spacing, TokenStream as TokenStream2, TokenTree};

use core_extensions::SelfOps;

struct WriteState {
    add_spacing_before: bool,
}

fn write_token_tree_inner(tt: TokenTree, s: &mut String, state: &mut WriteState) {
    let _added_spacing = mem::replace(&mut state.add_spacing_before, false);

    match tt {
        TokenTree::Group(group) => {
            let (start, end) = match group.delimiter() {
                Delimiter::Parenthesis => ('(', ')'),
                Delimiter::Brace => ('{', '}'),
                Delimiter::Bracket => ('[', ']'),
                Delimiter::None => (' ', ' '),
            };
            let _ = write!(s, "{} ", start);
            for nested_tt in group.stream() {
                write_token_tree(nested_tt, s);
            }
            state.add_spacing_before = false;
            let _ = write!(s, " {}", end);
        }
        TokenTree::Ident(x) => write!(s, "{} ", x).drop_(),
        TokenTree::Punct(punct) => {
            // if added_spacing {
            //     s.push(' ');
            // }
            s.push(punct.as_char());
            state.add_spacing_before = match punct.spacing() {
                Spacing::Alone => {
                    s.push(' ');
                    true
                }
                // Spacing::Alone=>true,
                Spacing::Joint => false,
            }
        }
        TokenTree::Literal(x) => write!(s, "{} ", x).drop_(),
    }
}

fn write_token_tree(tt: TokenTree, s: &mut String) {
    write_token_tree_inner(
        tt,
        s,
        &mut WriteState {
            add_spacing_before: false,
        },
    )
}

pub fn write_token_stream(ts: TokenStream2, buffer: &mut String) {
    for tt in ts {
        write_token_tree(tt, buffer);
    }
}

pub fn token_stream_to_string(ts: TokenStream2) -> String {
    let mut buffer = String::new();
    write_token_stream(ts, &mut buffer);
    buffer
}
