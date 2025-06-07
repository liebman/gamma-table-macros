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
}
