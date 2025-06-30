// auto_builder_derive/src/lib.rs
// Entry point for the AutoBuilder proc macro. Modules are split for clarity.

mod parse;
mod generator;

use proc_macro::TokenStream;

use crate::generator::expand_autobuilder;

#[proc_macro_derive(AutoBuilder, attributes(builder))]
pub fn auto_builder_derive(input: TokenStream) -> TokenStream {
    expand_autobuilder(input)
}
