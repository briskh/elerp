use libelp_proc_internal as internal;
use proc_macro::TokenStream;
// use quote::quote;
// use syn::LitStr;

use syn::parse_macro_input;

#[proc_macro_derive(Configuration, attributes(config))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    return internal::configure::handler(ast).into();
}

