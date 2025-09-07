use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Lit, LitStr, Type};
use syn::spanned::Spanned;

use super::{ConfigError, ConfigResult};

/// Process default value expression, check type matching and perform necessary conversions
pub fn process_default_value(
    default_expr: &Expr,
    field_ty: &Type,
) -> ConfigResult<TokenStream> {
    // Get field type name
    let type_name = get_type_name(field_ty)?;

    // Process literal expressions
    if let Expr::Lit(lit) = default_expr {
        match &lit.lit {
            Lit::Str(str_lit) => handle_string_literal(str_lit, &type_name),
            Lit::Int(int_lit) => handle_int_literal(int_lit, &type_name, field_ty),
            Lit::Float(float_lit) => handle_float_literal(float_lit, &type_name, field_ty),
            Lit::Bool(bool_lit) => handle_bool_literal(bool_lit, &type_name),
            _ => Err(ConfigError::unsupported_literal_type(
                &format!("{:?}", lit.lit),
                field_ty.span(),
            )),
        }
    } else if let Expr::Path(path_expr) = default_expr {
        // Process path expressions (such as identifiers)
        if let Some(segment) = path_expr.path.segments.last() {
            let ident = &segment.ident;
            match type_name.as_str() {
                // Treat identifier content as string literal
                "String" => {
                    let lit = LitStr::new(&ident.to_string(), Span::call_site());
                    Ok(quote! { #lit.to_string() })
                }
                "str" => {
                    let lit = LitStr::new(&ident.to_string(), Span::call_site());
                    Ok(quote! { #lit })
                }
                _ => Ok(quote! { #ident }),
            }
        } else {
            Err(ConfigError::cannot_parse_path_expression(default_expr.span()))
        }
    } else {
        // For other non-literal expressions, use directly
        Ok(quote! { #default_expr })
    }
}

/// Get type name
fn get_type_name(ty: &Type) -> ConfigResult<String> {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                Ok(segment.ident.to_string())
            } else {
                Err(ConfigError::cannot_identify_type_path(ty.span()))
            }
        }
        _ => Err(ConfigError::unsupported_type_format(ty.span())),
    }
}

/// Handle string literals
fn handle_string_literal(
    str_lit: &syn::LitStr,
    type_name: &str,
) -> ConfigResult<TokenStream> {
    match type_name {
        "String" => Ok(quote! { #str_lit.to_string() }),
        "str" => Ok(quote! { #str_lit }),
        _ => Err(ConfigError::string_literal_wrong_type(
            type_name,
            str_lit.span(),
        )),
    }
}

/// Handle integer literals
fn handle_int_literal(
    int_lit: &syn::LitInt,
    type_name: &str,
    field_ty: &Type,
) -> ConfigResult<TokenStream> {
    let value_str = int_lit.base10_digits();

    match type_name {
        "i8" => parse_and_quote::<i8>(value_str, field_ty),
        "i16" => parse_and_quote::<i16>(value_str, field_ty),
        "i32" => parse_and_quote::<i32>(value_str, field_ty),
        "i64" => parse_and_quote::<i64>(value_str, field_ty),
        "i128" => parse_and_quote::<i128>(value_str, field_ty),
        "isize" => parse_and_quote::<isize>(value_str, field_ty),
        "u8" => parse_and_quote::<u8>(value_str, field_ty),
        "u16" => parse_and_quote::<u16>(value_str, field_ty),
        "u32" => parse_and_quote::<u32>(value_str, field_ty),
        "u64" => parse_and_quote::<u64>(value_str, field_ty),
        "u128" => parse_and_quote::<u128>(value_str, field_ty),
        "usize" => parse_and_quote::<usize>(value_str, field_ty),
        _ => Err(ConfigError::integer_literal_wrong_type(
            type_name,
            field_ty.span(),
        )),
    }
}

/// Handle float literals
fn handle_float_literal(
    float_lit: &syn::LitFloat,
    type_name: &str,
    field_ty: &Type,
) -> ConfigResult<TokenStream> {
    let value_str = float_lit.base10_digits();

    match type_name {
        "f32" => parse_and_quote::<f32>(value_str, field_ty),
        "f64" => parse_and_quote::<f64>(value_str, field_ty),
        _ => Err(ConfigError::float_literal_wrong_type(
            type_name,
            field_ty.span(),
        )),
    }
}

/// Handle boolean literals
fn handle_bool_literal(
    bool_lit: &syn::LitBool,
    type_name: &str,
) -> ConfigResult<TokenStream> {
    match type_name {
        "bool" => Ok(quote! { #bool_lit }),
        _ => Err(ConfigError::boolean_literal_wrong_type(
            type_name,
            bool_lit.span(),
        )),
    }
}

/// Parse string to specified type and generate TokenStream
fn parse_and_quote<T>(value_str: &str, field_ty: &Type) -> ConfigResult<TokenStream>
where
    T: std::str::FromStr + quote::ToTokens,
    T::Err: std::fmt::Display,
{
    match value_str.parse::<T>() {
        Ok(parsed) => Ok(quote! { #parsed }),
        Err(e) => Err(ConfigError::parse_error(
            value_str,
            &std::any::type_name::<T>(),
            &e.to_string(),
            field_ty.span(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::{Expr, Lit, LitBool, LitFloat, LitInt, LitStr, Type, parse_quote};

    // Helper function: create type
    fn create_type(type_name: &str) -> Type {
        match type_name {
            "String" => parse_quote! { String },
            "str" => parse_quote! { str },
            "i8" => parse_quote! { i8 },
            "i16" => parse_quote! { i16 },
            "i32" => parse_quote! { i32 },
            "i64" => parse_quote! { i64 },
            "i128" => parse_quote! { i128 },
            "isize" => parse_quote! { isize },
            "u8" => parse_quote! { u8 },
            "u16" => parse_quote! { u16 },
            "u32" => parse_quote! { u32 },
            "u64" => parse_quote! { u64 },
            "u128" => parse_quote! { u128 },
            "usize" => parse_quote! { usize },
            "f32" => parse_quote! { f32 },
            "f64" => parse_quote! { f64 },
            "bool" => parse_quote! { bool },
            "MyType" => parse_quote! { MyType },
            "std::collections::HashMap" => parse_quote! { std::collections::HashMap },
            _ => parse_quote! { UnknownType },
        }
    }

    // Helper function: create string literal expression
    fn create_string_expr(value: &str) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Str(LitStr::new(value, proc_macro2::Span::call_site())),
        })
    }

    // Helper function: create integer literal expression
    fn create_int_expr(value: &str) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Int(LitInt::new(value, proc_macro2::Span::call_site())),
        })
    }

    // Helper function: create float literal expression
    fn create_float_expr(value: &str) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Float(LitFloat::new(value, proc_macro2::Span::call_site())),
        })
    }

    // Helper function: create boolean literal expression
    fn create_bool_expr(value: bool) -> Expr {
        Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Bool(LitBool::new(value, proc_macro2::Span::call_site())),
        })
    }

    // Helper function: create non-literal expression
    fn create_non_literal_expr() -> Expr {
        parse_quote! { some_function() }
    }

    #[test]
    fn test_process_default_value_string_to_string() {
        let expr = create_string_expr("hello");
        let ty = create_type("String");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { "hello".to_string() };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_string_to_str() {
        let expr = create_string_expr("world");
        let ty = create_type("str");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { "world" };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_string_to_wrong_type() {
        let expr = create_string_expr("hello");
        let ty = create_type("i32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("String literal cannot be used for type i32"));
    }

    #[test]
    fn test_process_default_value_int_to_i32() {
        let expr = create_int_expr("42");
        let ty = create_type("i32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        // Note: quote! generates code with type suffix, e.g., "42i32"
        let expected = quote! { 42i32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_int_to_u16() {
        let expr = create_int_expr("100");
        let ty = create_type("u16");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 100u16 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_int_to_wrong_type() {
        let expr = create_int_expr("42");
        let ty = create_type("String");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Integer literal cannot be used for type String"));
    }

    #[test]
    fn test_process_default_value_int_overflow() {
        let expr = create_int_expr("1000");
        let ty = create_type("i8");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Cannot parse '1000' as type"));
    }

    #[test]
    fn test_process_default_value_float_to_f32() {
        let expr = create_float_expr("3.14");
        let ty = create_type("f32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 3.14f32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_float_to_f64() {
        let expr = create_float_expr("2.718");
        let ty = create_type("f64");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 2.718f64 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_float_to_wrong_type() {
        let expr = create_float_expr("3.14");
        let ty = create_type("i32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Float literal cannot be used for type i32"));
    }

    #[test]
    fn test_process_default_value_bool_to_bool() {
        let expr = create_bool_expr(true);
        let ty = create_type("bool");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { true };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_bool_false() {
        let expr = create_bool_expr(false);
        let ty = create_type("bool");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { false };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_process_default_value_bool_to_wrong_type() {
        let expr = create_bool_expr(true);
        let ty = create_type("String");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(error.to_string().contains("Boolean literal cannot be used for type String"));
    }

    #[test]
    fn test_process_default_value_non_literal_expr() {
        let expr = create_non_literal_expr();
        let ty = create_type("MyType");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { some_function() };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_get_type_name_success() {
        let ty = create_type("String");
        let result = get_type_name(&ty);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "String");
    }

    #[test]
    fn test_get_type_name_complex_type() {
        let ty = create_type("std::collections::HashMap");
        let result = get_type_name(&ty);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "HashMap");
    }

    #[test]
    fn test_get_type_name_invalid_type() {
        let ty = parse_quote! { &str };
        let result = get_type_name(&ty);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported type format"));
    }

    #[test]
    fn test_handle_string_literal_string_type() {
        let str_lit = LitStr::new("test", proc_macro2::Span::call_site());
        let result = handle_string_literal(&str_lit, "String");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { "test".to_string() };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_string_literal_str_type() {
        let str_lit = LitStr::new("test", proc_macro2::Span::call_site());
        let result = handle_string_literal(&str_lit, "str");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { "test" };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_string_literal_wrong_type() {
        let str_lit = LitStr::new("test", proc_macro2::Span::call_site());
        let result = handle_string_literal(&str_lit, "i32");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("String literal cannot be used for type i32")
        );
    }

    #[test]
    fn test_handle_int_literal_all_types() {
        let int_lit = LitInt::new("42", proc_macro2::Span::call_site());
        let ty = create_type("i32");

        // Test all integer types
        let types = [
            "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        ];

        for type_name in &types {
            let result = handle_int_literal(&int_lit, type_name, &ty);
            assert!(result.is_ok(), "Failed for type: {}", type_name);
        }
    }

    #[test]
    fn test_handle_int_literal_wrong_type() {
        let int_lit = LitInt::new("42", proc_macro2::Span::call_site());
        let ty = create_type("String");
        let result = handle_int_literal(&int_lit, "String", &ty);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Integer literal cannot be used for type String")
        );
    }

    #[test]
    fn test_handle_float_literal_f32() {
        let float_lit = LitFloat::new("3.14", proc_macro2::Span::call_site());
        let ty = create_type("f32");
        let result = handle_float_literal(&float_lit, "f32", &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 3.14f32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_float_literal_f64() {
        let float_lit = LitFloat::new("2.718", proc_macro2::Span::call_site());
        let ty = create_type("f64");
        let result = handle_float_literal(&float_lit, "f64", &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 2.718f64 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_float_literal_wrong_type() {
        let float_lit = LitFloat::new("3.14", proc_macro2::Span::call_site());
        let ty = create_type("i32");
        let result = handle_float_literal(&float_lit, "i32", &ty);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Float literal cannot be used for type i32")
        );
    }

    #[test]
    fn test_handle_bool_literal_true() {
        let bool_lit = LitBool::new(true, proc_macro2::Span::call_site());
        let result = handle_bool_literal(&bool_lit, "bool");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { true };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_bool_literal_false() {
        let bool_lit = LitBool::new(false, proc_macro2::Span::call_site());
        let result = handle_bool_literal(&bool_lit, "bool");
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { false };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_handle_bool_literal_wrong_type() {
        let bool_lit = LitBool::new(true, proc_macro2::Span::call_site());
        let result = handle_bool_literal(&bool_lit, "String");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Boolean literal cannot be used for type String")
        );
    }

    #[test]
    fn test_parse_and_quote_success() {
        let ty = create_type("i32");
        let result = parse_and_quote::<i32>("42", &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 42i32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_parse_and_quote_failure() {
        let ty = create_type("i32");
        let result = parse_and_quote::<i32>("abc", &ty);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Cannot parse 'abc' as type")
        );
    }

    #[test]
    fn test_unsupported_literal_type() {
        // Create an unsupported character literal
        let expr = Expr::Lit(syn::ExprLit {
            attrs: vec![],
            lit: Lit::Char(syn::LitChar::new('a', proc_macro2::Span::call_site())),
        });
        let ty = create_type("String");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported literal type")
        );
    }

    #[test]
    fn test_edge_case_empty_string() {
        let expr = create_string_expr("");
        let ty = create_type("String");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { "".to_string() };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_edge_case_zero_int() {
        let expr = create_int_expr("0");
        let ty = create_type("i32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { 0i32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_edge_case_negative_int() {
        let expr = create_int_expr("-42");
        let ty = create_type("i32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { -42i32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }

    #[test]
    fn test_edge_case_negative_float() {
        let expr = create_float_expr("-3.14");
        let ty = create_type("f32");

        let result = process_default_value(&expr, &ty);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let expected = quote! { -3.14f32 };
        assert_eq!(tokens.to_string(), expected.to_string());
    }
}
