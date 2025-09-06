use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Type};

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

    // 用于计算最大嵌套深度的每个字段深度表达式
    let mut depth_exprs: Vec<proc_macro2::TokenStream> = Vec::new();

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

        // 生成深度表达式：基础类型 -> 0，其它类型 -> <T>::__ELP_DEPTH
        let field_ty = &field.ty;
        let is_primitive = match get_type_last_ident(field_ty).as_deref() {
            Some("String") | Some("str") | Some("i8") | Some("i16") | Some("i32") | Some("i64")
            | Some("i128") | Some("isize") | Some("u8") | Some("u16") | Some("u32")
            | Some("u64") | Some("u128") | Some("usize") | Some("f32") | Some("f64")
            | Some("bool") => true,
            _ => false,
        };
        if is_primitive {
            depth_exprs.push(quote! { 0usize });
        } else {
            depth_exprs.push(quote! { 1usize + <#field_ty>::__ELP_CHILD_DEPTH });
        }
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

    // 折叠求最大值表达式：(((0 max d1) max d2) ...)
    let mut max_fold: proc_macro2::TokenStream = quote! { 0usize };
    for de in depth_exprs {
        max_fold = quote! { Self::__elp_max(#max_fold, #de) };
    }

    let expanded = quote! {
        impl #name {
            pub fn new() -> Self {
                Self {
                    #(#field_assignments)*
                }
            }

            #from_toml_impl

            #to_toml_impl

            const fn __elp_max(a: usize, b: usize) -> usize { if a > b { a } else { b } }
            // 子结构体最大层数：基础类型=0，结构体=1+子层数
            pub const __ELP_CHILD_DEPTH: usize = { #max_fold };
            // 限制：最大两层（顶层+一层子结构体）；超过则给出友好的错误信息
            pub const __ELP_ASSERT_MSG: () = {
                if Self::__ELP_CHILD_DEPTH > 1 {
                    panic!(concat!(
                        "配置结构体 '",
                        stringify!(#name),
                        "' 嵌套层级超过允许的两层（顶层 + 一层子结构体）"
                    ));
                }
                ()
            };
            // 强制在类型层面引用上面的常量，确保编译期一定求值并报错
        pub const __ELP_ENFORCER: [(); { let _ = Self::__ELP_ASSERT_MSG; 1 }] = [(); { let _ = Self::__ELP_ASSERT_MSG; 1 }];
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }
    };

    return TokenStream::from(expanded);
}

// 获取类型最后一个标识符（类型名）
fn get_type_last_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}
