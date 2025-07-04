//! An internal crate for `gaussdb-types`.

#![recursion_limit = "256"]
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod accepts;
mod case;
mod composites;
mod enums;
mod fromsql;
mod overrides;
mod tosql;

#[proc_macro_derive(ToSql, attributes(gaussdb, postgres))]
pub fn derive_tosql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    tosql::expand_derive_tosql(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

#[proc_macro_derive(FromSql, attributes(gaussdb, postgres))]
pub fn derive_fromsql(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    fromsql::expand_derive_fromsql(input)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
