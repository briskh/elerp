use proc_macro2::Span;

/// Configuration processing errors
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unsupported literal type: {literal_type:?}")]
    UnsupportedLiteralType {
        literal_type: String,
        span: Span,
    },

    #[error("Cannot parse path expression at span")]
    CannotParsePathExpression {
        span: Span,
    },

    #[error("Cannot identify type path at span")]
    CannotIdentifyTypePath {
        span: Span,
    },

    #[error("Unsupported type format at span")]
    UnsupportedTypeFormat {
        span: Span,
    },

    #[error("String literal cannot be used for type {type_name}")]
    StringLiteralWrongType {
        type_name: String,
        span: Span,
    },

    #[error("Integer literal cannot be used for type {type_name}")]
    IntegerLiteralWrongType {
        type_name: String,
        span: Span,
    },

    #[error("Float literal cannot be used for type {type_name}")]
    FloatLiteralWrongType {
        type_name: String,
        span: Span,
    },

    #[error("Boolean literal cannot be used for type {type_name}")]
    BooleanLiteralWrongType {
        type_name: String,
        span: Span,
    },

    #[error("Cannot parse '{value}' as type {type_name}: {parse_error}")]
    ParseError {
        value: String,
        type_name: String,
        parse_error: String,
        span: Span,
    },

    #[error("Field must have a name")]
    FieldMustHaveName {
        span: Span,
    },

    #[error("This derive macro only supports structs")]
    OnlySupportsStructs {
        span: Span,
    },

    #[error("Configuration struct '{struct_name}' nesting level exceeds allowed two levels (top level + one level of nested structs)")]
    NestingLevelExceeded {
        struct_name: String,
        span: Span,
    },
}

impl ConfigError {
    /// Convert to syn::Error for proc macro compilation
    pub fn to_syn_error(&self) -> syn::Error {
        match self {
            ConfigError::UnsupportedLiteralType { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::CannotParsePathExpression { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::CannotIdentifyTypePath { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::UnsupportedTypeFormat { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::StringLiteralWrongType { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::IntegerLiteralWrongType { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::FloatLiteralWrongType { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::BooleanLiteralWrongType { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::ParseError { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::FieldMustHaveName { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::OnlySupportsStructs { span, .. } => {
                syn::Error::new(*span, self)
            }
            ConfigError::NestingLevelExceeded { span, .. } => {
                syn::Error::new(*span, self)
            }
        }
    }

    /// Create UnsupportedLiteralType error
    pub fn unsupported_literal_type(literal_type: &str, span: Span) -> Self {
        Self::UnsupportedLiteralType {
            literal_type: literal_type.to_string(),
            span,
        }
    }

    /// Create CannotParsePathExpression error
    pub fn cannot_parse_path_expression(span: Span) -> Self {
        Self::CannotParsePathExpression { span }
    }

    /// Create CannotIdentifyTypePath error
    pub fn cannot_identify_type_path(span: Span) -> Self {
        Self::CannotIdentifyTypePath { span }
    }

    /// Create UnsupportedTypeFormat error
    pub fn unsupported_type_format(span: Span) -> Self {
        Self::UnsupportedTypeFormat { span }
    }

    /// Create StringLiteralWrongType error
    pub fn string_literal_wrong_type(type_name: &str, span: Span) -> Self {
        Self::StringLiteralWrongType {
            type_name: type_name.to_string(),
            span,
        }
    }

    /// Create IntegerLiteralWrongType error
    pub fn integer_literal_wrong_type(type_name: &str, span: Span) -> Self {
        Self::IntegerLiteralWrongType {
            type_name: type_name.to_string(),
            span,
        }
    }

    /// Create FloatLiteralWrongType error
    pub fn float_literal_wrong_type(type_name: &str, span: Span) -> Self {
        Self::FloatLiteralWrongType {
            type_name: type_name.to_string(),
            span,
        }
    }

    /// Create BooleanLiteralWrongType error
    pub fn boolean_literal_wrong_type(type_name: &str, span: Span) -> Self {
        Self::BooleanLiteralWrongType {
            type_name: type_name.to_string(),
            span,
        }
    }

    /// Create ParseError error
    pub fn parse_error(value: &str, type_name: &str, parse_error: &str, span: Span) -> Self {
        Self::ParseError {
            value: value.to_string(),
            type_name: type_name.to_string(),
            parse_error: parse_error.to_string(),
            span,
        }
    }

    /// Create FieldMustHaveName error
    pub fn field_must_have_name(span: Span) -> Self {
        Self::FieldMustHaveName { span }
    }

    /// Create OnlySupportsStructs error
    pub fn only_supports_structs(span: Span) -> Self {
        Self::OnlySupportsStructs { span }
    }

    /// Create NestingLevelExceeded error
    pub fn nesting_level_exceeded(struct_name: &str, span: Span) -> Self {
        Self::NestingLevelExceeded {
            struct_name: struct_name.to_string(),
            span,
        }
    }
}

/// Result type alias for configuration processing
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Helper trait to convert ConfigError to syn::Error
pub trait ToSynError {
    fn to_syn_error(self) -> syn::Error;
}

impl ToSynError for ConfigError {
    fn to_syn_error(self) -> syn::Error {
        ConfigError::to_syn_error(&self)
    }
}

impl ToSynError for syn::Error {
    fn to_syn_error(self) -> syn::Error {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::Span;

    // Helper function to create a test span
    fn test_span() -> Span {
        Span::call_site()
    }

    // Test error creation and basic properties
    #[test]
    fn test_unsupported_literal_type_creation() {
        let span = test_span();
        let error = ConfigError::unsupported_literal_type("char", span);
        
        match error {
            ConfigError::UnsupportedLiteralType { literal_type, span: _error_span } => {
                assert_eq!(literal_type, "char");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected UnsupportedLiteralType variant"),
        }
    }

    #[test]
    fn test_cannot_parse_path_expression_creation() {
        let span = test_span();
        let error = ConfigError::cannot_parse_path_expression(span);
        
        match error {
            ConfigError::CannotParsePathExpression { span: _error_span } => {
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected CannotParsePathExpression variant"),
        }
    }

    #[test]
    fn test_cannot_identify_type_path_creation() {
        let span = test_span();
        let error = ConfigError::cannot_identify_type_path(span);
        
        match error {
            ConfigError::CannotIdentifyTypePath { span: _error_span } => {
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected CannotIdentifyTypePath variant"),
        }
    }

    #[test]
    fn test_unsupported_type_format_creation() {
        let span = test_span();
        let error = ConfigError::unsupported_type_format(span);
        
        match error {
            ConfigError::UnsupportedTypeFormat { span: _error_span } => {
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected UnsupportedTypeFormat variant"),
        }
    }

    #[test]
    fn test_string_literal_wrong_type_creation() {
        let span = test_span();
        let error = ConfigError::string_literal_wrong_type("i32", span);
        
        match error {
            ConfigError::StringLiteralWrongType { type_name, span: _error_span } => {
                assert_eq!(type_name, "i32");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected StringLiteralWrongType variant"),
        }
    }

    #[test]
    fn test_integer_literal_wrong_type_creation() {
        let span = test_span();
        let error = ConfigError::integer_literal_wrong_type("String", span);
        
        match error {
            ConfigError::IntegerLiteralWrongType { type_name, span: _error_span } => {
                assert_eq!(type_name, "String");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected IntegerLiteralWrongType variant"),
        }
    }

    #[test]
    fn test_float_literal_wrong_type_creation() {
        let span = test_span();
        let error = ConfigError::float_literal_wrong_type("bool", span);
        
        match error {
            ConfigError::FloatLiteralWrongType { type_name, span: _error_span } => {
                assert_eq!(type_name, "bool");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected FloatLiteralWrongType variant"),
        }
    }

    #[test]
    fn test_boolean_literal_wrong_type_creation() {
        let span = test_span();
        let error = ConfigError::boolean_literal_wrong_type("f64", span);
        
        match error {
            ConfigError::BooleanLiteralWrongType { type_name, span: _error_span } => {
                assert_eq!(type_name, "f64");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected BooleanLiteralWrongType variant"),
        }
    }

    #[test]
    fn test_parse_error_creation() {
        let span = test_span();
        let error = ConfigError::parse_error("abc", "i32", "invalid digit", span);
        
        match error {
            ConfigError::ParseError { value, type_name, parse_error, span: _error_span } => {
                assert_eq!(value, "abc");
                assert_eq!(type_name, "i32");
                assert_eq!(parse_error, "invalid digit");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_field_must_have_name_creation() {
        let span = test_span();
        let error = ConfigError::field_must_have_name(span);
        
        match error {
            ConfigError::FieldMustHaveName { span: _error_span } => {
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected FieldMustHaveName variant"),
        }
    }

    #[test]
    fn test_only_supports_structs_creation() {
        let span = test_span();
        let error = ConfigError::only_supports_structs(span);
        
        match error {
            ConfigError::OnlySupportsStructs { span: _error_span } => {
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected OnlySupportsStructs variant"),
        }
    }

    #[test]
    fn test_nesting_level_exceeded_creation() {
        let span = test_span();
        let error = ConfigError::nesting_level_exceeded("MyConfig", span);
        
        match error {
            ConfigError::NestingLevelExceeded { struct_name, span: _error_span } => {
                assert_eq!(struct_name, "MyConfig");
                // Note: Span doesn't implement PartialEq, so we can't compare spans directly
            }
            _ => panic!("Expected NestingLevelExceeded variant"),
        }
    }

    // Test error message formatting
    #[test]
    fn test_unsupported_literal_type_message() {
        let error = ConfigError::unsupported_literal_type("char", test_span());
        let message = error.to_string();
        assert!(message.contains("Unsupported literal type"));
        assert!(message.contains("char"));
    }

    #[test]
    fn test_cannot_parse_path_expression_message() {
        let error = ConfigError::cannot_parse_path_expression(test_span());
        let message = error.to_string();
        assert!(message.contains("Cannot parse path expression"));
    }

    #[test]
    fn test_cannot_identify_type_path_message() {
        let error = ConfigError::cannot_identify_type_path(test_span());
        let message = error.to_string();
        assert!(message.contains("Cannot identify type path"));
    }

    #[test]
    fn test_unsupported_type_format_message() {
        let error = ConfigError::unsupported_type_format(test_span());
        let message = error.to_string();
        assert!(message.contains("Unsupported type format"));
    }

    #[test]
    fn test_string_literal_wrong_type_message() {
        let error = ConfigError::string_literal_wrong_type("i32", test_span());
        let message = error.to_string();
        assert!(message.contains("String literal cannot be used for type i32"));
    }

    #[test]
    fn test_integer_literal_wrong_type_message() {
        let error = ConfigError::integer_literal_wrong_type("String", test_span());
        let message = error.to_string();
        assert!(message.contains("Integer literal cannot be used for type String"));
    }

    #[test]
    fn test_float_literal_wrong_type_message() {
        let error = ConfigError::float_literal_wrong_type("bool", test_span());
        let message = error.to_string();
        assert!(message.contains("Float literal cannot be used for type bool"));
    }

    #[test]
    fn test_boolean_literal_wrong_type_message() {
        let error = ConfigError::boolean_literal_wrong_type("f64", test_span());
        let message = error.to_string();
        assert!(message.contains("Boolean literal cannot be used for type f64"));
    }

    #[test]
    fn test_parse_error_message() {
        let error = ConfigError::parse_error("abc", "i32", "invalid digit", test_span());
        let message = error.to_string();
        assert!(message.contains("Cannot parse 'abc' as type i32: invalid digit"));
    }

    #[test]
    fn test_field_must_have_name_message() {
        let error = ConfigError::field_must_have_name(test_span());
        let message = error.to_string();
        assert!(message.contains("Field must have a name"));
    }

    #[test]
    fn test_only_supports_structs_message() {
        let error = ConfigError::only_supports_structs(test_span());
        let message = error.to_string();
        assert!(message.contains("This derive macro only supports structs"));
    }

    #[test]
    fn test_nesting_level_exceeded_message() {
        let error = ConfigError::nesting_level_exceeded("MyConfig", test_span());
        let message = error.to_string();
        assert!(message.contains("Configuration struct 'MyConfig' nesting level exceeds allowed two levels"));
    }

    // Test to_syn_error conversion
    #[test]
    fn test_unsupported_literal_type_to_syn_error() {
        let span = test_span();
        let error = ConfigError::unsupported_literal_type("char", span);
        let syn_error = error.to_syn_error();
        
        // Verify it's a syn::Error
        assert!(syn_error.to_compile_error().to_string().contains("Unsupported literal type"));
    }

    #[test]
    fn test_cannot_parse_path_expression_to_syn_error() {
        let span = test_span();
        let error = ConfigError::cannot_parse_path_expression(span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Cannot parse path expression"));
    }

    #[test]
    fn test_cannot_identify_type_path_to_syn_error() {
        let span = test_span();
        let error = ConfigError::cannot_identify_type_path(span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Cannot identify type path"));
    }

    #[test]
    fn test_unsupported_type_format_to_syn_error() {
        let span = test_span();
        let error = ConfigError::unsupported_type_format(span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Unsupported type format"));
    }

    #[test]
    fn test_string_literal_wrong_type_to_syn_error() {
        let span = test_span();
        let error = ConfigError::string_literal_wrong_type("i32", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("String literal cannot be used for type i32"));
    }

    #[test]
    fn test_integer_literal_wrong_type_to_syn_error() {
        let span = test_span();
        let error = ConfigError::integer_literal_wrong_type("String", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Integer literal cannot be used for type String"));
    }

    #[test]
    fn test_float_literal_wrong_type_to_syn_error() {
        let span = test_span();
        let error = ConfigError::float_literal_wrong_type("bool", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Float literal cannot be used for type bool"));
    }

    #[test]
    fn test_boolean_literal_wrong_type_to_syn_error() {
        let span = test_span();
        let error = ConfigError::boolean_literal_wrong_type("f64", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Boolean literal cannot be used for type f64"));
    }

    #[test]
    fn test_parse_error_to_syn_error() {
        let span = test_span();
        let error = ConfigError::parse_error("abc", "i32", "invalid digit", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Cannot parse 'abc' as type i32: invalid digit"));
    }

    #[test]
    fn test_field_must_have_name_to_syn_error() {
        let span = test_span();
        let error = ConfigError::field_must_have_name(span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Field must have a name"));
    }

    #[test]
    fn test_only_supports_structs_to_syn_error() {
        let span = test_span();
        let error = ConfigError::only_supports_structs(span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("This derive macro only supports structs"));
    }

    #[test]
    fn test_nesting_level_exceeded_to_syn_error() {
        let span = test_span();
        let error = ConfigError::nesting_level_exceeded("MyConfig", span);
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Configuration struct 'MyConfig' nesting level exceeds allowed two levels"));
    }

    // Test ToSynError trait implementation
    #[test]
    fn test_config_error_to_syn_error_trait() {
        let error = ConfigError::unsupported_literal_type("char", test_span());
        let syn_error = error.to_syn_error();
        
        assert!(syn_error.to_compile_error().to_string().contains("Unsupported literal type"));
    }

    #[test]
    fn test_syn_error_to_syn_error_trait() {
        let original_error = syn::Error::new(test_span(), "test error");
        let original_message = original_error.to_compile_error().to_string();
        let converted_error = original_error.to_syn_error();
        
        assert_eq!(converted_error.to_compile_error().to_string(), original_message);
    }

    // Test ConfigResult type alias
    #[test]
    fn test_config_result_ok() {
        let result: ConfigResult<i32> = Ok(42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_config_result_err() {
        let error = ConfigError::parse_error("abc", "i32", "invalid", test_span());
        let result: ConfigResult<i32> = Err(error);
        
        match result {
            Err(ConfigError::ParseError { value, type_name, .. }) => {
                assert_eq!(value, "abc");
                assert_eq!(type_name, "i32");
            }
            _ => panic!("Expected ParseError"),
        }
    }

    // Test Debug implementation
    #[test]
    fn test_debug_implementation() {
        let error = ConfigError::unsupported_literal_type("char", test_span());
        let debug_str = format!("{:?}", error);
        
        // Debug should contain the variant name
        assert!(debug_str.contains("UnsupportedLiteralType"));
    }

    // Test error trait implementation
    #[test]
    fn test_error_trait_implementation() {
        let error = ConfigError::parse_error("abc", "i32", "invalid", test_span());
        
        // Should implement std::error::Error
        let error_ref: &dyn std::error::Error = &error;
        assert!(error_ref.to_string().contains("Cannot parse 'abc' as type i32: invalid"));
    }

    // Test edge cases and special characters in error messages
    #[test]
    fn test_special_characters_in_error_messages() {
        let error = ConfigError::parse_error("test\nvalue", "MyType<T>", "error with \"quotes\"", test_span());
        let message = error.to_string();
        
        assert!(message.contains("test\nvalue"));
        assert!(message.contains("MyType<T>"));
        assert!(message.contains("error with \"quotes\""));
    }

    #[test]
    fn test_empty_strings_in_error_messages() {
        let error = ConfigError::parse_error("", "", "", test_span());
        let message = error.to_string();
        
        assert!(message.contains("Cannot parse '' as type : "));
    }

    #[test]
    fn test_unicode_in_error_messages() {
        let error = ConfigError::parse_error("测试", "类型", "错误", test_span());
        let message = error.to_string();
        
        assert!(message.contains("测试"));
        assert!(message.contains("类型"));
        assert!(message.contains("错误"));
    }

    // Test all error variants in to_syn_error method
    #[test]
    fn test_all_variants_to_syn_error_coverage() {
        let span = test_span();
        
        // Test all variants to ensure 100% coverage of the match statement
        let errors = vec![
            ConfigError::UnsupportedLiteralType { literal_type: "char".to_string(), span },
            ConfigError::CannotParsePathExpression { span },
            ConfigError::CannotIdentifyTypePath { span },
            ConfigError::UnsupportedTypeFormat { span },
            ConfigError::StringLiteralWrongType { type_name: "i32".to_string(), span },
            ConfigError::IntegerLiteralWrongType { type_name: "String".to_string(), span },
            ConfigError::FloatLiteralWrongType { type_name: "bool".to_string(), span },
            ConfigError::BooleanLiteralWrongType { type_name: "f64".to_string(), span },
            ConfigError::ParseError { 
                value: "abc".to_string(), 
                type_name: "i32".to_string(), 
                parse_error: "invalid".to_string(), 
                span 
            },
            ConfigError::FieldMustHaveName { span },
            ConfigError::OnlySupportsStructs { span },
            ConfigError::NestingLevelExceeded { struct_name: "MyConfig".to_string(), span },
        ];

        for error in errors {
            let syn_error = error.to_syn_error();
            // Just verify it doesn't panic and returns a syn::Error
            assert!(!syn_error.to_compile_error().to_string().is_empty());
        }
    }

    // Test span preservation
    #[test]
    fn test_span_preservation() {
        let span1 = Span::call_site();
        let span2 = Span::call_site();
        
        let error1 = ConfigError::unsupported_literal_type("char", span1);
        let error2 = ConfigError::unsupported_literal_type("char", span2);
        
        match (error1, error2) {
            (
                ConfigError::UnsupportedLiteralType { span: _s1, .. },
                ConfigError::UnsupportedLiteralType { span: _s2, .. }
            ) => {
                // Spans should be preserved (but we can't compare them directly due to PartialEq)
                // The fact that we can destructure them means they are preserved
            }
            _ => panic!("Expected UnsupportedLiteralType variants"),
        }
    }

    // Test error message consistency
    #[test]
    fn test_error_message_consistency() {
        let error1 = ConfigError::string_literal_wrong_type("i32", test_span());
        let error2 = ConfigError::string_literal_wrong_type("i32", test_span());
        
        assert_eq!(error1.to_string(), error2.to_string());
    }

    // Test error with different spans but same content
    #[test]
    fn test_error_with_different_spans() {
        let span1 = Span::call_site();
        let span2 = Span::call_site();
        
        let error1 = ConfigError::parse_error("abc", "i32", "invalid", span1);
        let error2 = ConfigError::parse_error("abc", "i32", "invalid", span2);
        
        // Error messages should be the same (span doesn't affect message)
        assert_eq!(error1.to_string(), error2.to_string());
    }
}