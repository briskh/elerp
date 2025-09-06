use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Configuration, attributes(default, note))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let data = match &ast.data {
        // 检查是否为结构体,如果为结构体,则继续处理
        syn::Data::Struct(s) => s,
        // 如果为其他类型,则返回错误
        _ => {
            return syn::Error::new_spanned(
                name,
                "Configuration derive macro only supports structs",
            )
            .to_compile_error()
            .into();
        }
    };

    for field in &data.fields {
      for attr in &field.attrs {
        println!("{:?}", attr);
      }
    }


    let expanded = quote! {
      impl #name {
        pub fn hello() -> &'static str { "hello from derive" }
      }
    };
    TokenStream::from(expanded)
}
