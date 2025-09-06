use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr};

use darling::FromField;

mod process_default_value;

#[derive(FromField, Default, Debug)]
#[darling(attributes(config))]
struct ConfigurationField {
    #[darling(default)]
    default: Option<Expr>,
    #[darling(default)]
    note: Option<String>,
}

pub fn handler(ast: DeriveInput) -> TokenStream {
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
    // let mut inits = Vec::new();

    for field in &data.fields {
        let ident = field.ident.as_ref().expect("named fields only");
        let opts = ConfigurationField::from_field(field).unwrap_or_default();
        let default_value = match opts.default {
            Some(ref default_expr) => {
                // 使用独立的默认值处理模块
                match process_default_value::process_default_value(default_expr, &field.ty) {
                    Ok(parsed_token) => {
                        println!("成功处理默认值: {:?} -> {:?}", default_expr, parsed_token);
                        parsed_token
                    }
                    Err(e) => {
                        // 如果处理失败，返回编译错误
                        return e.to_compile_error().into();
                    }
                }
            }
            None => {
                // 没有默认值，使用 Default::default()
                quote! { Default::default() }
            }
        };

        println!("字段: {:?}", ident);
        println!("类型: {:?}", field.ty);
        println!("注释: {:?}", opts.note);
        println!("默认值: {:?}", default_value);
    }

    let expanded = quote! {
      impl #name {
        pub fn hello() -> &'static str { "hello from derive macro" }
      }
    };
    return TokenStream::from(expanded);
}
