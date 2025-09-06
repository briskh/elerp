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
                    // 对于其他类型，尝试从字符串解析
                    toml_value.get(stringify!(#field_name))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<#field_type>().ok())
                        .unwrap_or_default()
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
    let mut field_blocks = Vec::new();

    for (field, default_expr, note) in field_configs {
        let field_name = field
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(field, "字段必须有名称"))?;

        let type_name = get_type_name(&field.ty)?;
        let note_text = note.as_deref().unwrap_or("");
        let default_value = field_value_to_toml_string(field, default_expr, &field.ty)?;

        // 生成用于比较的默认值表达式（类型安全）
        let default_compare_tokens: TokenStream = if let Some(expr) = default_expr {
            // 复用默认值处理逻辑，得到一个可直接比较的表达式 Token
            match super::process_default_value::process_default_value(expr, &field.ty) {
                Ok(ts) => ts,
                Err(e) => return Err(e),
            }
        } else {
            quote! { Default::default() }
        };

        // 生成每个字段的TOML块
        let field_block = match type_name.as_str() {
            "String" => {
                quote! {
                    // 注释行
                    format!("#{}: {}, default: {}", #note_text, #type_name, #default_value),
                    // 字段行（使用 toml::to_string 生成 key = value）
                    {
                        let __is_default = self.#field_name == #default_compare_tokens;
                        let __cur_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            __m.insert(stringify!(#field_name).to_string(), self.#field_name.clone());
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        let __default_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            let __default_value = #default_compare_tokens;
                            __m.insert(stringify!(#field_name).to_string(), __default_value);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        if __is_default { format!("# {}", __default_line) } else { __cur_line }
                    },
                    String::new(),
                }
            }
            "i32" | "i64" | "u16" | "u32" | "u64" | "usize" | "isize" | "i8" | "i16" | "u8"
            | "i128" | "u128" => {
                quote! {
                    format!("#{}: {}, default: {}", #note_text, #type_name, #default_value),
                    {
                        let __is_default = self.#field_name == #default_compare_tokens;
                        let __cur_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            __m.insert(stringify!(#field_name).to_string(), self.#field_name);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        let __default_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            let __default_value = #default_compare_tokens;
                            __m.insert(stringify!(#field_name).to_string(), __default_value);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        if __is_default { format!("# {}", __default_line) } else { __cur_line }
                    },
                    String::new(),
                }
            }
            "f64" | "f32" => {
                quote! {
                    format!("#{}: {}, default: {}", #note_text, #type_name, #default_value),
                    {
                        let __is_default = self.#field_name == #default_compare_tokens;
                        let __cur_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            __m.insert(stringify!(#field_name).to_string(), self.#field_name);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        let __default_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            let __default_value = #default_compare_tokens;
                            __m.insert(stringify!(#field_name).to_string(), __default_value);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        if __is_default { format!("# {}", __default_line) } else { __cur_line }
                    },
                    String::new(),
                }
            }
            "bool" => {
                quote! {
                    format!("#{}: {}, default: {}", #note_text, #type_name, #default_value),
                    {
                        let __is_default = self.#field_name == #default_compare_tokens;
                        let __cur_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            __m.insert(stringify!(#field_name).to_string(), self.#field_name);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        let __default_line = {
                            let mut __m = ::std::collections::BTreeMap::new();
                            let __default_value = #default_compare_tokens;
                            __m.insert(stringify!(#field_name).to_string(), __default_value);
                            ::toml::to_string(&__m).unwrap_or_default().trim_end().to_string()
                        };
                        if __is_default { format!("# {}", __default_line) } else { __cur_line }
                    },
                    String::new(),
                }
            }
            _ => {
                quote! {
                    format!("#{}: {}, default: {}", #note_text, #type_name, #default_value),
                    {
                        let mut __m = ::toml::map::Map::new();
                        let __default_line = {
                            let mut __m = ::toml::map::Map::new();
                            let __default_value = #default_compare_tokens;
                            __m.insert(stringify!(#field_name).to_string(), ::toml::Value::from(__default_value));
                            ::toml::Value::Table(__m).to_string().trim_end().to_string()
                        };
                        format!("# {}", __default_line)
                    },
                    String::new(),
                }
            }
        };

        field_blocks.push(field_block);
    }

    Ok(quote! {
        pub fn to_toml(&self) -> String {
            let mut lines = Vec::new();

            #(lines.extend(vec![#field_blocks]);)*

            lines.join("\n")
        }
    })
}
