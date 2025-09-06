use super::*;
use quote::quote;
use syn::{parse_quote, DeriveInput, Expr, Field, Type};
use syn::{FieldMutability};
use syn::token::Colon;

// 工具：构建带字段的结构体 DeriveInput
fn make_struct(struct_name: &str, fields: Vec<(&str, Type, Option<Expr>, Option<&str>)>) -> DeriveInput {
    // 使用 syn 构造一个结构体 token 再 parse 为 DeriveInput 更稳妥
    let mut field_tokens: Vec<proc_macro2::TokenStream> = Vec::new();
    for (name, ty, default_expr, note) in fields {
        let ident: syn::Ident = syn::Ident::new(name, proc_macro2::Span::call_site());
        let ty_ts = ty.clone();
        let mut attrs: Vec<syn::Attribute> = Vec::new();
        // 组装 #[config(...)]
        if let Some(expr) = default_expr {
            let ts = quote! { #expr }.to_string();
            // 直接文本插入会带引号/后缀，使用 parse_quote! 简化
            let attr: syn::Attribute = parse_quote! { #[config(default = #expr)] };
            attrs.push(attr);
        }
        if let Some(n) = note {
            let lit: syn::LitStr = syn::LitStr::new(n, proc_macro2::Span::call_site());
            let attr: syn::Attribute = parse_quote! { #[config(note = #lit)] };
            attrs.push(attr);
        }
        let field: Field = Field {
            attrs,
            vis: parse_quote! { pub },
            mutability: FieldMutability::None,
            ident: Some(ident),
            colon_token: Some(Colon::default()),
            ty: ty_ts,
        };
        field_tokens.push(quote! { #field });
    }

    let struct_ident: syn::Ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
    let tokens = quote! {
        struct #struct_ident { #( #field_tokens , )* }
    };
    syn::parse2(tokens).unwrap()
}

#[test]
fn test_get_type_last_ident_primitives_and_struct() {
    let ty: Type = parse_quote! { i32 };
    assert_eq!(get_type_last_ident(&ty).as_deref(), Some("i32"));
    let ty: Type = parse_quote! { std::string::String };
    assert_eq!(get_type_last_ident(&ty).as_deref(), Some("String"));
    let ty: Type = parse_quote! { MyStruct };
    assert_eq!(get_type_last_ident(&ty).as_deref(), Some("MyStruct"));
    let ty: Type = parse_quote! { &str };
    assert_eq!(get_type_last_ident(&ty), None);
}

#[test]
fn test_handler_non_struct_error() {
    // enum 不支持，应返回编译错误 token
    let di: DeriveInput = parse_quote! {
        enum E { A, B }
    };
    let ts = handler(di);
    let s = ts.to_string();
    assert!(s.contains("only supports structs") || s.contains("this derive macro only supports structs"));
}

#[test]
fn test_handler_generates_impls_for_primitives_and_nested() {
    // 顶层包含基础类型与一个子结构体，验证 __ELP_CHILD_DEPTH、new、to_toml/from_toml 生成
    let child: DeriveInput = make_struct(
        "Child",
        vec![
            ("host", parse_quote! { String }, Some(parse_quote! { "localhost" }), Some("redis host")),
            ("port", parse_quote! { u16 }, Some(parse_quote! { 6379 }), Some("redis port")),
        ],
    );
    let _ = handler(child); // 只需生成，不断言文本

    let parent: DeriveInput = make_struct(
        "Parent",
        vec![
            ("feature", parse_quote! { bool }, Some(parse_quote! { true }), Some("enable")),
            ("child", parse_quote! { Child }, None, None),
        ],
    );
    let ts_parent = handler(parent);
    let s = ts_parent.to_string();
    // 检查关键符号存在
    assert!(s.contains("impl Parent"));
    assert!(s.contains("pub fn new"));
    assert!(s.contains("pub fn from_toml"));
    assert!(s.contains("pub fn to_toml"));
    assert!(s.contains("__ELP_CHILD_DEPTH"));
    assert!(s.contains("__ELP_ASSERT_MSG"));
}

#[test]
fn test_handler_depth_assert_over_two_levels_panics() {
    // 三层嵌套：Top -> Mid -> Leaf，应在常量计算中包含 panic 文本
    let leaf: DeriveInput = make_struct(
        "Leaf",
        vec![("v", parse_quote! { i32 }, Some(parse_quote! { 1 }), None)],
    );
    let _ = handler(leaf);

    let mid: DeriveInput = make_struct(
        "Mid",
        vec![("leaf", parse_quote! { Leaf }, None, None)],
    );
    let _ = handler(mid);

    let top: DeriveInput = make_struct(
        "Top",
        vec![("mid", parse_quote! { Mid }, None, None)],
    );
    let ts = handler(top);
    let s = ts.to_string();
    assert!(s.contains("嵌套层级超过允许的两层") || s.contains("__ELP_ASSERT_MSG"));
}

#[test]
fn test_toml_utils_field_value_and_generate_impls() {
    // 验证 field_value_to_toml_string 对默认值和类型的分支；同时间接覆盖 generate_to_toml_impl
    let field: Field = parse_quote! { #[config(default = "hi", note = "n")] pub name: String };
    let ty = field.ty.clone();
    let s = super::toml_utils::field_value_to_toml_string(&field, &Some(parse_quote! { "hi" }), &ty)
        .unwrap();
    assert_eq!(s, "\"hi\"");

    let field_num: Field = parse_quote! { #[config(default = 10)] pub n: u32 };
    let s = super::toml_utils::field_value_to_toml_string(&field_num, &Some(parse_quote! { 10 }), &field_num.ty)
        .unwrap();
    assert_eq!(s, "10");

    let field_bool: Field = parse_quote! { #[config(default = true)] pub b: bool };
    let s = super::toml_utils::field_value_to_toml_string(&field_bool, &Some(parse_quote! { true }), &field_bool.ty)
        .unwrap();
    assert_eq!(s, "true");

    // 生成 to_toml / from_toml 代码
    let di: DeriveInput = make_struct(
        "Cfg",
        vec![
            ("name", parse_quote! { String }, Some(parse_quote! { "hi" }), Some("note")),
            ("n", parse_quote! { u32 }, Some(parse_quote! { 10 }), Some("num")),
            ("b", parse_quote! { bool }, Some(parse_quote! { true }), Some("flag")),
        ],
    );
    let ts = handler(di);
    let s = ts.to_string();
    assert!(s.contains("from_toml"));
    assert!(s.contains("to_toml"));
}

#[test]
fn test_toml_utils_field_value_expr_path_for_string_and_int() {
    let field_s: Field = parse_quote! { #[config(default = localhost)] pub host: String };
    let s = super::toml_utils::field_value_to_toml_string(&field_s, &Some(parse_quote! { localhost }), &field_s.ty)
        .unwrap();
    assert_eq!(s, "\"localhost\"");

    let field_n: Field = parse_quote! { #[config(default = TEN)] pub n: u32 };
    let s = super::toml_utils::field_value_to_toml_string(&field_n, &Some(parse_quote! { TEN }), &field_n.ty)
        .unwrap();
    assert_eq!(s, "TEN");
}

#[test]
fn test_toml_utils_field_value_default_none_by_type_and_non_literal_expr() {
    let field_s: Field = parse_quote! { pub s: String };
    assert_eq!(super::toml_utils::field_value_to_toml_string(&field_s, &None, &field_s.ty).unwrap(), "\"\"");

    let field_i: Field = parse_quote! { pub i: i32 };
    assert_eq!(super::toml_utils::field_value_to_toml_string(&field_i, &None, &field_i.ty).unwrap(), "0");

    let field_f: Field = parse_quote! { pub f: f64 };
    assert_eq!(super::toml_utils::field_value_to_toml_string(&field_f, &None, &field_f.ty).unwrap(), "0.0");

    let field_b: Field = parse_quote! { pub b: bool };
    assert_eq!(super::toml_utils::field_value_to_toml_string(&field_b, &None, &field_b.ty).unwrap(), "false");

    let field_u: Field = parse_quote! { pub x: UnknownType };
    assert_eq!(super::toml_utils::field_value_to_toml_string(&field_u, &None, &field_u.ty).unwrap(), "null");

    // 非字面量表达式 -> default_value
    let field_s2: Field = parse_quote! { pub a: String };
    let s = super::toml_utils::field_value_to_toml_string(&field_s2, &Some(parse_quote!{ some_call() }), &field_s2.ty)
        .unwrap();
    assert_eq!(s, "default_value");
}

#[test]
fn test_generate_from_toml_impl_contains_expected_branches() {
    let f_string: Field = parse_quote! { pub s: String };
    let f_i32: Field = parse_quote! { pub a: i32 };
    let f_i64: Field = parse_quote! { pub b: i64 };
    let f_u16: Field = parse_quote! { pub c: u16 };
    let f_u32: Field = parse_quote! { pub d: u32 };
    let f_u64: Field = parse_quote! { pub e: u64 };
    let f_f64: Field = parse_quote! { pub g: f64 };
    let f_bool: Field = parse_quote! { pub h: bool };
    let f_nested: Field = parse_quote! { pub sub: Sub };

    let fields: Vec<&Field> = vec![
        &f_string, &f_i32, &f_i64, &f_u16, &f_u32, &f_u64, &f_f64, &f_bool, &f_nested
    ];
    let ts = super::toml_utils::generate_from_toml_impl(&parse_quote! { Test }, &fields);
    let s = ts.to_string();
    assert!(s.contains("as_str"));
    assert!(s.contains("as_integer"));
    assert!(s.contains("as_float"));
    assert!(s.contains("as_bool"));
    assert!(s.contains("from_toml"));
}

#[test]
fn test_generate_to_toml_impl_error_on_unnamed_field() {
    // 构造无 ident 的字段（元组字段）
    let unnamed: Field = parse_quote! { #[config(default = 1)] u32 };
    let res = super::toml_utils::generate_to_toml_impl(
        &parse_quote! { S },
        &[],
        &[(unnamed, Some(parse_quote! { 1 }), Some("n".to_string()))],
    );
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert!(msg.contains("字段必须有名称"));
}

#[test]
fn test_handler_error_from_invalid_default_literal() {
    // 对 String 使用字符字面量，触发 process_default_value 的错误并在 handler 里冒泡为编译错误 token
    let di: DeriveInput = make_struct(
        "Bad",
        vec![
            ("s", parse_quote! { String }, Some(parse_quote! { 'a' }), Some("bad")),
        ],
    );
    let ts = handler(di);
    let s = ts.to_string();
    assert!(s.contains("不支持的字面量类型") || s.contains("compile_error"));
}

