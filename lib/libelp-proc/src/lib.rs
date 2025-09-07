use libelp_proc_internal as internal;
use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Derive macro for configuration structs
/// 
/// This macro generates configuration handling code for structs, including:
/// - `new()` method with default values
/// - `from_toml()` method for parsing TOML configuration
/// - `to_toml()` method for generating TOML configuration
/// - Compile-time validation of configuration structure
/// 
/// # Example
/// 
/// ```rust
/// use libelp::Configuration;
/// 
/// #[derive(Configuration)]
/// struct AppConfig {
///     #[config(default = "localhost", note = "Server hostname")]
///     host: String,
///     #[config(default = 8080, note = "Server port")]
///     port: u16,
///     #[config(default = true, note = "Enable debug mode")]
///     debug: bool,
/// }
/// ```
/// 
/// # Errors
/// 
/// This macro will generate compilation errors for:
/// - Non-struct types (only structs are supported)
/// - Invalid default value types
/// - Configuration nesting exceeding 2 levels
/// - Other configuration validation errors
#[proc_macro_derive(Configuration, attributes(config))]
pub fn derive_configuration(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    
    // Call the internal handler which returns a TokenStream
    // If there are errors, they will be converted to compilation errors
    internal::configure::handler(ast).into()
}

