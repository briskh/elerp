use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Field, Lit, Type};

/// 将字段值转换为TOML字符串表示
pub fn field_value_to_toml_string(
    _field: &Field,
    default_expr: &Option<Expr>,
    field_ty: &Type,
) -> Result<String, syn::Error> {
    let type_name = get_type_name(field_ty)?;

    if let Some(default_expr) = default_expr {
        if let Expr::Lit(lit) = default_expr {
            match &lit.lit {
                Lit::Str(str_lit) => Ok(format!("\"{}\"", str_lit.value())),
                Lit::Int(int_lit) => Ok(int_lit.base10_digits().to_string()),
                Lit::Float(float_lit) => Ok(float_lit.base10_digits().to_string()),
                Lit::Bool(bool_lit) => Ok(bool_lit.value().to_string()),
                _ => Err(syn::Error::new_spanned(
                    field_ty,
                    format!("不支持的字面量类型: {:?}", lit.lit),
                )),
            }
        } else if let Expr::Path(path_expr) = default_expr {
            // 当字符串默认值被解析为路径标识符时（如 localhost）
            if let Some(segment) = path_expr.path.segments.last() {
                let ident_text = segment.ident.to_string();
                match type_name.as_str() {
                    "String" | "str" => Ok(format!("\"{}\"", ident_text)),
                    _ => Ok(ident_text),
                }
            } else {
                Ok("default_value".to_string())
            }
        } else {
            // 对于其他非字面量表达式，返回占位符
            Ok("default_value".to_string())
        }
    } else {
        // 没有默认值，根据类型返回默认的TOML表示
        match type_name.as_str() {
            "String" => Ok("\"\"".to_string()),
            "str" => Ok("\"\"".to_string()),
            "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
            | "u128" | "usize" => Ok("0".to_string()),
            "f32" | "f64" => Ok("0.0".to_string()),
            "bool" => Ok("false".to_string()),
            _ => Ok("null".to_string()),
        }
    }
}

/// 获取类型名称
fn get_type_name(ty: &Type) -> Result<String, syn::Error> {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                Ok(segment.ident.to_string())
            } else {
                Err(syn::Error::new_spanned(ty, "无法识别类型路径"))
            }
        }
        _ => Err(syn::Error::new_spanned(ty, "不支持的类型格式")),
    }
}

/// 生成from_toml方法的实现
pub fn generate_from_toml_impl(_struct_name: &syn::Ident, fields: &[&Field]) -> TokenStream {
    let field_assignments: Vec<TokenStream> = fields
        .iter()
        .filter_map(|field| {
            let field_name = field.ident.as_ref()?;
            let field_type = &field.ty;
            let type_name = get_type_name(field_type).ok()?;

            // 根据类型生成不同的解析逻辑
            let parse_logic = match type_name.as_str() {
                "String" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                },
                "i32" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_integer())
                        .and_then(|i| i.try_into().ok())
                        .unwrap_or_default()
                },
                "i64" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_integer())
                        .unwrap_or_default()
                },
                "u16" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_integer())
                        .and_then(|i| i.try_into().ok())
                        .unwrap_or_default()
                },
                "u32" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_integer())
                        .and_then(|i| i.try_into().ok())
                        .unwrap_or_default()
                },
                "u64" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_integer())
                        .and_then(|i| i.try_into().ok())
                        .unwrap_or_default()
                },
                "f64" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_float())
                        .unwrap_or_default()
                },
                "bool" => quote! {
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_bool())
                        .unwrap_or_default()
                },
                _ => quote! {
                    // 嵌套结构体（最多两层）：从子表递归解析
                    match toml_value.get(stringify!(#field_name)).and_then(|v| v.as_table()) {
                        Some(tbl) => {
                            let sub_str = ::toml::to_string(tbl).unwrap_or_default();
                            <#field_type>::from_toml(&sub_str).unwrap_or_default()
                        }
                        None => Default::default(),
                    }
                },
            };

            Some(quote! {
                #field_name: #parse_logic,
            })
        })
        .collect();

    quote! {
        pub fn from_toml(toml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
            let toml_value: toml::Value = toml::from_str(toml_str)?;

            Ok(Self {
                #(#field_assignments)*
            })
        }
    }
}

/// 生成to_toml方法的实现
pub fn generate_to_toml_impl(
    _struct_name: &syn::Ident,
    _fields: &[&Field],
    field_configs: &[(Field, Option<Expr>, Option<String>)],
) -> Result<TokenStream, syn::Error> {
    // 为每个字段生成分支代码
    let mut per_field_snippets: Vec<TokenStream> = Vec::new();

    for (field, default_expr, note) in field_configs {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "字段必须有名称"))?;
        let type_name = get_type_name(&field.ty)?;
        let note_text = note.as_deref().unwrap_or("");
        let default_value_lit = field_value_to_toml_string(field, default_expr, &field.ty)?;

        let default_compare_tokens: TokenStream = if let Some(expr) = default_expr {
            match super::process_default_value::process_default_value(expr, &field.ty) {
                Ok(ts) => ts,
                Err(e) => return Err(e),
            }
        } else {
            quote! { Default::default() }
        };

        let is_primitive = matches!(
            type_name.as_str(),
            "String"
                | "str"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
                | "bool"
        );

        let snippet = if is_primitive {
            quote! {
                // 注释
                lines.push(format!("# {}, {}, 默认值: {}", #note_text, #type_name, #default_value_lit));
                // 值行
                {
                    let __is_default = self.#field_ident == #default_compare_tokens;
                    let __line = {
                        let mut __m = ::std::collections::BTreeMap::new();
                        __m.insert(stringify!(#field_ident).to_string(), self.#field_ident.clone());
                        ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                    };
                    if __is_default { lines.push(format!("# {}", __line)); } else { lines.push(__line); }
                }
                lines.push(String::new());
            }
        } else {
            // 嵌套结构体：最多两层。深度为0时添加段名；深度>=1时直接拼接子内容（其内部会自行截止到两层）。
            quote! {
                if __depth == 0 {
                    lines.push(format!("[{}]", stringify!(#field_ident)));
                }
                lines.push(self.#field_ident.__elp_to_toml_depth(__depth + 1));
                lines.push(String::new());
            }
        };

        per_field_snippets.push(snippet);
    }

    let expanded = quote! {
        pub fn to_toml(&self) -> String { self.__elp_to_toml_depth(0) }

        pub fn __elp_to_toml_depth(&self, __depth: usize) -> String {
            let mut lines: ::std::vec::Vec<::std::string::String> = Vec::new();
            #(#per_field_snippets)*
            lines.join("\n")
        }
    };

    Ok(expanded)
}
