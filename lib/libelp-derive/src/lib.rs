use proc_macro::TokenStream;
use syn::parse_macro_input;

use libelp_derive_internal;

#[proc_macro_derive(Configuration, attributes(default, note))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    return libelp_derive_internal::derive_configuration_handler(ast).into();
}
