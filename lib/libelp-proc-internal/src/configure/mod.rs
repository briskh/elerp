use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr};

use darling::FromField;

mod process_default_value;
mod toml_utils;

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

    // 收集字段配置信息
    let mut field_configs = Vec::new();
    let mut field_assignments = Vec::new();

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

        // 收集字段配置信息用于TOML生成
        field_configs.push((field.clone(), opts.default.clone(), opts.note.clone()));

        // 生成字段赋值
        field_assignments.push(quote! {
            #ident: #default_value,
        });

        println!("字段: {:?}", ident);
        println!("类型: {:?}", field.ty);
        println!("注释: {:?}", opts.note);
        println!("默认值: {:?}", default_value);
    }

    // 收集字段引用
    let fields: Vec<&syn::Field> = data.fields.iter().collect();

    // 生成from_toml方法
    let from_toml_impl = toml_utils::generate_from_toml_impl(name, &fields);

    // 生成to_toml方法
    let to_toml_impl = match toml_utils::generate_to_toml_impl(name, &fields, &field_configs) {
        Ok(impl_code) => impl_code,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#field_assignments)*
                }
            }

            #from_toml_impl

            #to_toml_impl
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    return TokenStream::from(expanded);
}
