# Error Handling with thiserror

This library uses the [thiserror](https://docs.rs/thiserror/) crate to provide structured error handling for configuration processing. This document explains the error handling patterns and best practices used in this codebase.

## Error Types

The main error type is `ConfigError`, which is an enum that covers all possible configuration processing errors:

```rust
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unsupported literal type: {literal_type:?}")]
    UnsupportedLiteralType { literal_type: String, span: Span },
    
    #[error("Cannot parse path expression at span")]
    CannotParsePathExpression { span: Span },
    
    #[error("String literal cannot be used for type {type_name}")]
    StringLiteralWrongType { type_name: String, span: Span },
    
    // ... more error variants
}
```

## Key Features

### 1. Structured Error Information
Each error variant includes:
- **Descriptive error messages** using thiserror's `#[error]` attribute
- **Context information** (e.g., type names, values, spans)
- **Source location** (proc_macro2::Span) for precise error reporting

### 2. Automatic Error Conversion
The `ToSynError` trait provides seamless conversion to `syn::Error` for proc macro compilation:

```rust
pub trait ToSynError {
    fn to_syn_error(self) -> syn::Error;
}
```

### 3. Result Type Alias
A convenient type alias for cleaner code:

```rust
pub type ConfigResult<T> = Result<T, ConfigError>;
```

## Usage Patterns

### Creating Errors
Use the provided constructor methods for consistent error creation:

```rust
// Type mismatch error
let error = ConfigError::string_literal_wrong_type("i32", span);

// Parse error with context
let error = ConfigError::parse_error("invalid", "i32", "invalid digit", span);

// Unsupported literal type
let error = ConfigError::unsupported_literal_type("char", span);
```

### Error Propagation
Errors automatically convert to `syn::Error` when needed:

```rust
match process_default_value(expr, field_ty) {
    Ok(token) => token,
    Err(e) => return e.to_syn_error().to_compile_error().into(),
}
```

### Error Handling in Tests
Test error conditions by checking error messages:

```rust
#[test]
fn test_invalid_type() {
    let result = process_default_value(&expr, &ty);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("String literal cannot be used for type i32"));
}
```

## Best Practices

### 1. Include Context Information
Always include relevant context in error messages:
- Type names for type mismatch errors
- Values for parse errors
- Source spans for precise error location

### 2. Use Descriptive Error Messages
Error messages should be:
- **Clear and actionable** - tell the user what went wrong and how to fix it
- **Consistent** - use similar phrasing across similar error types
- **Informative** - include relevant context without being verbose

### 3. Preserve Source Location
Always include the `Span` information for proc macro errors to enable precise error reporting in the user's code.

### 4. Test Error Conditions
Write comprehensive tests for error conditions:
- Test all error variants
- Verify error messages contain expected information
- Test error conversion to `syn::Error`

## Error Categories

### 1. Type System Errors
- `StringLiteralWrongType` - String literal used for non-string type
- `IntegerLiteralWrongType` - Integer literal used for non-integer type
- `FloatLiteralWrongType` - Float literal used for non-float type
- `BooleanLiteralWrongType` - Boolean literal used for non-boolean type

### 2. Parsing Errors
- `ParseError` - Failed to parse a value as a specific type
- `CannotParsePathExpression` - Invalid path expression syntax
- `UnsupportedLiteralType` - Literal type not supported

### 3. Type Resolution Errors
- `CannotIdentifyTypePath` - Cannot extract type name from path
- `UnsupportedTypeFormat` - Type format not supported

### 4. Structural Errors
- `FieldMustHaveName` - Field without identifier (tuple field)
- `OnlySupportsStructs` - Non-struct type used with derive macro
- `NestingLevelExceeded` - Configuration nesting exceeds allowed levels

## Integration with thiserror

The error handling system leverages thiserror's features:

- **Automatic Display implementation** via `#[error]` attributes
- **Source chain support** for error chaining
- **Debug implementation** for debugging
- **Error trait implementation** for standard error handling

This provides a robust, user-friendly error handling system that integrates well with Rust's error handling ecosystem while providing precise error reporting for proc macro users.
