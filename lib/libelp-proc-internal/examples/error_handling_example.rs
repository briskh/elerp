//! Example demonstrating the error handling system with thiserror
//! 
//! This example shows how the configuration processing errors are handled
//! using the thiserror library for better error messages and debugging.

use libelp_proc_internal::configure::{ConfigError, ConfigResult};

/// Example function that demonstrates error handling
fn process_config_value(value: &str) -> ConfigResult<i32> {
    match value.parse::<i32>() {
        Ok(v) => Ok(v),
        Err(e) => Err(ConfigError::parse_error(
            value,
            "i32",
            &e.to_string(),
            proc_macro2::Span::call_site(),
        )),
    }
}

/// Example function that demonstrates different error types
fn demonstrate_errors() {
    // Example of parsing error
    match process_config_value("invalid") {
        Ok(v) => println!("Parsed value: {}", v),
        Err(e) => println!("Parse error: {}", e),
    }

    // Example of type mismatch error
    let error = ConfigError::string_literal_wrong_type(
        "i32",
        proc_macro2::Span::call_site(),
    );
    println!("Type mismatch error: {}", error);

    // Example of unsupported literal type
    let error = ConfigError::unsupported_literal_type(
        "char",
        proc_macro2::Span::call_site(),
    );
    println!("Unsupported literal error: {}", error);

    // Example of nesting level exceeded
    let error = ConfigError::nesting_level_exceeded(
        "MyConfig",
        proc_macro2::Span::call_site(),
    );
    println!("Nesting level error: {}", error);
}

fn main() {
    println!("=== Error Handling Example ===");
    demonstrate_errors();
    
    println!("\n=== Successful Parsing ===");
    match process_config_value("42") {
        Ok(v) => println!("Successfully parsed: {}", v),
        Err(e) => println!("Error: {}", e),
    }
}
