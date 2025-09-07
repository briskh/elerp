use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Type};

use darling::FromField;

mod error;
mod process_default_value;
mod toml_utils;

pub use error::{ConfigError, ConfigResult, ToSynError};

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
        // Check if it's a struct, if so, continue processing
        syn::Data::Struct(s) => s,
        // If it's another type, return error
        _ => {
            return ConfigError::only_supports_structs(name.span())
                .to_syn_error()
                .to_compile_error()
                .into();
        }
    };

    // Collect field configuration information
    let mut field_configs = Vec::new();
    let mut field_assignments = Vec::new();

    // Depth expression for each field to calculate maximum nesting depth
    let mut depth_exprs: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in &data.fields {
        let ident = field.ident.as_ref().expect("named fields only");
        let opts = ConfigurationField::from_field(field).unwrap_or_default();

        let default_value = match opts.default {
            Some(ref default_expr) => {
                // Use independent default value processing module
                match process_default_value::process_default_value(default_expr, &field.ty) {
                    Ok(parsed_token) => parsed_token,
                    Err(e) => {
                        // If processing fails, return compile error
                        return e.to_syn_error().to_compile_error().into();
                    }
                }
            }
            None => {
                // No default value, use Default::default()
                quote! { Default::default() }
            }
        };

        // Collect field configuration information for TOML generation
        field_configs.push((field.clone(), opts.default.clone(), opts.note.clone()));

        // Generate field assignment
        field_assignments.push(quote! {
            #ident: #default_value,
        });


        // Generate depth expression: basic types -> 0, other types -> <T>::__ELP_DEPTH
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

    // Collect field references
    let fields: Vec<&syn::Field> = data.fields.iter().collect();

    // --- Auto-impl serde Serialize/Deserialize ---
    // Build helper struct fields for Serialize (borrowed fields)
    let ser_helper_fields: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("named fields only");
            let ty = &f.ty;
            quote! { #ident: &'__elp_a #ty, }
        })
        .collect();

    // Initialize helper for Serialize
    let ser_helper_inits: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("named fields only");
            quote! { #ident: &self.#ident, }
        })
        .collect();

    let serialize_impl = quote! {
        impl serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                #[derive(serde::Serialize)]
                struct __ElpSerdeHelper<'__elp_a> { #( #ser_helper_fields )* }
                let helper = __ElpSerdeHelper { #( #ser_helper_inits )* };
                helper.serialize(serializer)
            }
        }
    };

    // Build helper struct fields for Deserialize (owned fields)
    let de_helper_fields: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("named fields only");
            let ty = &f.ty;
            quote! { #ident: #ty, }
        })
        .collect();

    // Reconstruct Self from helper
    let de_self_inits: Vec<proc_macro2::TokenStream> = fields
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("named fields only");
            quote! { #ident: helper.#ident, }
        })
        .collect();

    let deserialize_impl = quote! {
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde::Deserialize)]
                struct __ElpSerdeHelper { #( #de_helper_fields )* }
                let helper = __ElpSerdeHelper::deserialize(deserializer)?;
                Ok(Self { #( #de_self_inits )* })
            }
        }
    };

    // Generate from_toml method
    let from_toml_impl = toml_utils::generate_from_toml_impl(name, &fields);

    // Generate to_toml method
    let to_toml_impl = match toml_utils::generate_to_toml_impl(name, &fields, &field_configs) {
        Ok(impl_code) => impl_code,
        Err(e) => return e.to_syn_error().to_compile_error().into(),
    };

    // Fold to find maximum expression: (((0 max d1) max d2) ...)
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
            // Maximum depth of child structs: basic types=0, structs=1+child depth
            pub const __ELP_CHILD_DEPTH: usize = { #max_fold };
            // Limit: maximum two levels (top level + one level of nested structs); provide friendly error message if exceeded
            pub const __ELP_ASSERT_MSG: () = {
                if Self::__ELP_CHILD_DEPTH > 1 {
                    panic!(concat!(
                        "Configuration struct '",
                        stringify!(#name),
                        "' nesting level exceeds allowed two levels (top level + one level of nested structs)"
                    ));
                }
                ()
            };
            // Force reference to the above constant at type level to ensure compile-time evaluation and error reporting
        pub const __ELP_ENFORCER: [(); { let _ = Self::__ELP_ASSERT_MSG; 1 }] = [(); { let _ = Self::__ELP_ASSERT_MSG; 1 }];
        }

        impl Default for #name {
            fn default() -> Self {
                Self::new()
            }
        }

        #serialize_impl
        #deserialize_impl

        impl libelp::Configuration for #name {
            fn new() -> Self {
                Self::new()
            }
        }
    };

    return TokenStream::from(expanded);
}

// Get the last identifier of a type (type name)
fn get_type_last_ident(ty: &Type) -> Option<String> {
    if let Type::Path(tp) = ty {
        tp.path.segments.last().map(|s| s.ident.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests;
