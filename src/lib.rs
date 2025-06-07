/// Procedural macro for generating gamma lookup tables.
///
/// By default uses gamma encoding (input^gamma).
/// When decoding=true, uses gamma correction/decoding (input^(1/gamma)).
///
/// # Parameters
/// - `name`: The name of the const table to be generated
/// - `entry_type`: The unsigned integer type of each entry (u8, u16, u32, u64)
/// - `gamma`: The gamma value (float)
/// - `size`: Number of table entries (minimum 3)
/// - `max_value`: Maximum output value to limit brightness (optional, defaults to size-1)
/// - `decoding`: Use gamma correction/decoding instead of encoding (optional, defaults to false)
///
/// # Examples
///
/// Gamma encoding (default):
/// ```
/// use gamma_table_macros::gamma_table;
///
/// gamma_table! {
///     name: GAMMA_ENCODED_TABLE,
///     entry_type: u8,
///     gamma: 2.2,
///     size: 256
/// }
/// ```
///
/// Gamma correction/decoding:
/// ```
/// use gamma_table_macros::gamma_table;
///
/// gamma_table! {
///     name: GAMMA_DECODED_TABLE,
///     entry_type: u8,
///     gamma: 2.2,
///     size: 256,
///     decoding: true
/// }
/// ```
extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Error, LitBool, LitFloat, LitInt};

#[proc_macro]
pub fn gamma_table(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as GammaTableInput);

    match generate_gamma_table(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct GammaTableInput {
    name: syn::Ident,
    entry_type: syn::Type,
    gamma: f64,
    size: usize,
    max_value: Option<u64>,
    decoding: Option<bool>,
}

impl syn::parse::Parse for GammaTableInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut name = None;
        let mut entry_type = None;
        let mut gamma = None;
        let mut size = None;
        let mut max_value = None;
        let mut decoding = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![:]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    let value: syn::Ident = input.parse()?;
                    name = Some(value);
                }
                "entry_type" => {
                    let value: syn::Type = input.parse()?;
                    entry_type = Some(value);
                }
                "gamma" => {
                    let value: LitFloat = input.parse()?;
                    gamma = Some(value.base10_parse()?);
                }
                "size" => {
                    let value: LitInt = input.parse()?;
                    size = Some(value.base10_parse()?);
                }
                "max_value" => {
                    let value: LitInt = input.parse()?;
                    max_value = Some(value.base10_parse()?);
                }
                "decoding" => {
                    let value: LitBool = input.parse()?;
                    decoding = Some(value.value);
                }
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("Unknown parameter: {}", ident),
                    ))
                }
            }

            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }

        Ok(GammaTableInput {
            name: name
                .ok_or_else(|| Error::new(input.span(), "Missing required parameter: name"))?,
            entry_type: entry_type.ok_or_else(|| {
                Error::new(input.span(), "Missing required parameter: entry_type")
            })?,
            gamma: gamma
                .ok_or_else(|| Error::new(input.span(), "Missing required parameter: gamma"))?,
            size: size
                .ok_or_else(|| Error::new(input.span(), "Missing required parameter: size"))?,
            max_value,
            decoding,
        })
    }
}

fn get_integer_type_max_value(entry_type: &syn::Type) -> Option<u64> {
    // Extract the type name from syn::Type
    if let syn::Type::Path(type_path) = entry_type {
        if let Some(segment) = type_path.path.segments.last() {
            match segment.ident.to_string().as_str() {
                "u8" => Some(u8::MAX as u64),
                "u16" => Some(u16::MAX as u64),
                "u32" => Some(u32::MAX as u64),
                "u64" => Some(u64::MAX),
                _ => None, // Unknown or unsupported type
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn generate_gamma_table(input: GammaTableInput) -> syn::Result<TokenStream> {
    let name = &input.name;
    let entry_type = &input.entry_type;
    let gamma = input.gamma;
    let size = input.size;
    let max_value = input.max_value.unwrap_or((size - 1) as u64);
    let decoding = input.decoding.unwrap_or(false);

    // Validate input parameters
    if gamma <= 0.0 {
        return Err(Error::new(name.span(), "Gamma value must be positive"));
    }
    if size < 3 {
        return Err(Error::new(
            name.span(),
            "Size must be at least 3 to create a meaningful gamma table. Smaller sizes only have min and max values.",
        ));
    }

    // Validate that max_value fits in the target integer type
    if let Some(type_max) = get_integer_type_max_value(entry_type) {
        if max_value > type_max {
            return Err(Error::new(
                name.span(),
                format!(
                    "max_value ({}) exceeds the maximum value ({}) that can be stored in entry_type {}",
                    max_value,
                    type_max,
                    quote!(#entry_type)
                ),
            ));
        }
    } else {
        return Err(Error::new(
            name.span(),
            format!("Unsupported entry_type: {}. Supported types are: u8, u16, u32, u64", quote!(#entry_type)),
        ));
    }

    // Generate the lookup table values
    let values = generate_table_values(size, gamma, max_value, decoding)?;

    // Convert values to tokens with proper casting
    let value_tokens: Vec<TokenStream> = values
        .iter()
        .map(|&v| quote! { #v as #entry_type })
        .collect();

    Ok(quote! {
        const #name: [#entry_type; #size] = [#(#value_tokens),*];
    })
}

fn generate_table_values(
    size: usize,
    gamma: f64,
    max_value: u64,
    decoding: bool,
) -> syn::Result<Vec<u64>> {
    let mut values = Vec::with_capacity(size);

    // Choose gamma exponent based on mode
    let gamma_exponent = if decoding {
        1.0 / gamma // Gamma correction/decoding: input^(1/gamma)
    } else {
        gamma // Gamma encoding (default): input^gamma
    };

    // Direct gamma processing for each entry
    for i in 0..size {
        let normalized_input = i as f64 / (size - 1) as f64;
        let processed = normalized_input.powf(gamma_exponent);
        let output_value = (processed * max_value as f64).round() as u64;
        values.push(output_value.min(max_value));
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gamma_encoding_default() {
        // Test gamma encoding (default behavior)
        let values = generate_table_values(256, 2.2, 255, false).unwrap();
        assert_eq!(values.len(), 256);
        assert_eq!(values[0], 0);
        assert_eq!(values[255], 255);

        // Values should be monotonically increasing
        for i in 1..values.len() {
            assert!(values[i] >= values[i - 1]);
        }
    }

    #[test]
    fn test_gamma_decoding() {
        // Test gamma correction/decoding
        let values = generate_table_values(256, 2.2, 255, true).unwrap();
        assert_eq!(values.len(), 256);
        assert_eq!(values[0], 0);
        assert_eq!(values[255], 255);

        // Values should be monotonically increasing
        for i in 1..values.len() {
            assert!(values[i] >= values[i - 1]);
        }
    }

    #[test]
    fn test_encoding_vs_decoding_difference() {
        let encoding_values = generate_table_values(10, 2.2, 100, false).unwrap();
        let decoding_values = generate_table_values(10, 2.2, 100, true).unwrap();

        // Encoding and decoding should produce different results for mid-values
        assert_ne!(encoding_values[5], decoding_values[5]);

        // But endpoints should be the same
        assert_eq!(encoding_values[0], decoding_values[0]); // Both 0
        assert_eq!(encoding_values[9], decoding_values[9]); // Both 100
    }

    #[test]
    fn test_default_max_value() {
        // Test that max_value defaults to size-1
        let values = generate_table_values(10, 1.0, 9, false).unwrap();
        assert_eq!(values[0], 0);
        assert_eq!(values[9], 9); // size-1
    }

    #[test]
    fn test_minimum_size_validation() {
        // Test that size must be at least 3
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: 2.2,
            size: 2,
            max_value: None,
            decoding: None,
        };

        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Size must be at least 3"));

        // Test that size 3 works
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: 2.2,
            size: 3,
            max_value: None,
            decoding: None,
        };

        let result = generate_gamma_table(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_negative_gamma_validation() {
        // Test that gamma must be positive
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: -1.0,
            size: 10,
            max_value: None,
            decoding: None,
        };

        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Gamma value must be positive"));

        // Test that gamma = 0 is also invalid
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: 0.0,
            size: 10,
            max_value: None,
            decoding: None,
        };

        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Gamma value must be positive"));
    }

    #[test]
    fn test_parsing_unknown_parameter() {
        // Test unknown parameter error
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10,
            unknown_param: 42
        };
        
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parsing_missing_required_parameters() {
        // Test missing name
        let tokens: proc_macro2::TokenStream = quote! {
            entry_type: u8,
            gamma: 2.2,
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());

        // Test missing entry_type
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            gamma: 2.2,
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());

        // Test missing gamma
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());

        // Test missing size
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_parsing_invalid_parameter_types() {
        // Test sending wrong types for each parameter
        
        // name expects IDENT, sending int
        let tokens: proc_macro2::TokenStream = quote! {
            name: 123,
            entry_type: u8,
            gamma: 2.2,
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // entry_type expects Type, sending string
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: "u8",
            gamma: 2.2,
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // gamma expects LitFloat, sending string
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: "2.2",
            size: 10
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // size expects LitInt, sending float
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10.5
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // size expects LitInt, sending string
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: "10"
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // max_value expects LitInt, sending float
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10,
            max_value: 255.5
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // max_value expects LitInt, sending string
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10,
            max_value: "255"
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // decoding expects LitBool, sending float
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10,
            decoding: 1.0
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
        
        // decoding expects LitBool, sending string
        let tokens: proc_macro2::TokenStream = quote! {
            name: TEST_TABLE,
            entry_type: u8,
            gamma: 2.2,
            size: 10,
            decoding: "true"
        };
        let result = syn::parse2::<GammaTableInput>(tokens);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_value_overflow_validation() {
        // Test u8 overflow
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(300), // Exceeds u8::MAX (255)
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_value (300) exceeds the maximum value (255)"));

        // Test u16 overflow
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u16").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(70000), // Exceeds u16::MAX (65535)
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_value (70000) exceeds the maximum value (65535)"));

        // Test u32 overflow
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u32").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(5000000000), // Exceeds u32::MAX (4294967295)
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("max_value (5000000000) exceeds the maximum value (4294967295)"));

        // Test valid max_value for u8
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u8").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(255), // Valid for u8
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_ok());

        // Test valid max_value for u32
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u32").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(1000000), // Valid for u32
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_ok());

        // Test valid max_value for u64
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("u64").unwrap(),
            gamma: 2.2,
            size: 10,
            max_value: Some(1000000), // Valid for u64
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_ok());

        // Test unsupported entry type
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("i32").unwrap(), // Unsupported type
            gamma: 2.2,
            size: 10,
            max_value: Some(100),
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported entry_type"));

        // Test another unsupported entry type
        let input = GammaTableInput {
            name: syn::parse_str("TEST_TABLE").unwrap(),
            entry_type: syn::parse_str("f32").unwrap(), // Unsupported type
            gamma: 2.2,
            size: 10,
            max_value: Some(100),
            decoding: None,
        };
        let result = generate_gamma_table(input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unsupported entry_type"));
    }
}
