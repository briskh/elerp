use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub fn handle_derive_configuration(ast: DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let data = match &ast.data {
        // 检查是否为结构体,如果为结构体,则继续处理
        syn::Data::Struct(s) => s,
        // 如果为其他类型,则返回错误
        _ => {
            return syn::Error::new_spanned(name, "this derive macro only supports structs")
                .to_compile_error()
                .into();
        }
    };

    for field in &data.fields {
        for attr in &field.attrs {
            println!("field attr: {:?}", attr);
        }
    }

    let expanded = quote! {
      impl #name {
        pub fn hello() -> &'static str { "hello from derive macro" }
      }
    };
    TokenStream::from(expanded)
}
