extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn serde_yojson_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let data = match input.data {
        Data::Enum(data) => data,
        _ => panic!("SerdeYojsonEnum can only be applied to enums"),
    };

    // We define this to be able to derive serde's defaults to use in the binary encoding
    let binary_enum_name = syn::Ident::new(&format!("Binary{}", name), name.span());

    let binary_variants = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let fields = &variant.fields;

        match fields {
            Fields::Named(_) | Fields::Unnamed(_) | Fields::Unit => {
                quote! {
                    #variant_ident #fields,
                }
            }
        }
    });

    let binary_enum_definition = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        enum #binary_enum_name {
            #( #binary_variants )*
        }
    };

    let variant_matches = data
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = to_snake_case(&variant_ident.to_string());

            match &variant.fields {
                Fields::Named(ref fields) => {
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    quote! {
                        #name::#variant_ident { #( ref #field_names ),* } => {
                            if serializer.is_human_readable() {
                                let mut seq = serializer.serialize_tuple(2)?;
                                seq.serialize_element(#variant_name)?;
                                seq.serialize_element(&serde_json::json!({
                                    #( stringify!(#field_names): #field_names ),*
                                }))?;
                                seq.end()
                            } else {
                                let binary_version = #binary_enum_name::#variant_ident { #( #field_names: #field_names.clone() ),* };
                                binary_version.serialize(serializer)
                            }
                        }
                    }
                }
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let field = &fields.unnamed[0];
                    // TODO(tizoc): only for polymorphic variants tuples should be spliced
                    // Check if the unnamed field is a tuple type and splice it
                    if let syn::Type::Tuple(ref tuple) = field.ty {
                        let tuple_len = tuple.elems.len();
                        let tuple_pattern: Vec<_> = (0..tuple_len)
                            .map(|i| syn::Ident::new(&format!("elem{}", i), proc_macro2::Span::call_site()))
                            .collect();

                        quote! {
                            #name::#variant_ident(( #( ref #tuple_pattern ),* )) => {
                                if serializer.is_human_readable() {
                                    let mut seq = serializer.serialize_tuple(#tuple_len + 1)?;
                                    seq.serialize_element(#variant_name)?;
                                    #( seq.serialize_element(#tuple_pattern)?; )*
                                    seq.end()
                                } else {
                                    let binary_version = #binary_enum_name::#variant_ident(( #( #tuple_pattern.clone() ),* ));
                                    binary_version.serialize(serializer)
                                }
                            }
                        }
                    } else {
                        quote! {
                            #name::#variant_ident(ref value) => {
                                if serializer.is_human_readable() {
                                    let mut seq = serializer.serialize_tuple(2)?;
                                    seq.serialize_element(#variant_name)?;
                                    seq.serialize_element(value)?;
                                    seq.end()
                                } else {
                                    let binary_version = #binary_enum_name::#variant_ident(value.clone());
                                    binary_version.serialize(serializer)
                                }
                            }
                        }
                    }
                }
                Fields::Unit => {
                    quote! {
                        #name::#variant_ident => {
                            if serializer.is_human_readable() {
                                let mut seq = serializer.serialize_tuple(1)?;
                                seq.serialize_element(#variant_name)?;
                                seq.end()
                            } else {
                                let binary_version = #binary_enum_name::#variant_ident;
                                binary_version.serialize(serializer)
                            }
                        }
                    }
                }
                _ => panic!("SerdeYojsonEnum only supports unit, single-value tuple, and struct-like variants"),
            }
        });

    let variant_deserialize_matches = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_name = to_snake_case(&variant_ident.to_string());

        match &variant.fields {
            Fields::Named(ref fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                quote! {
                    #variant_name => {
                        let map = seq.next_element::<serde_json::Value>()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                        let map = map.as_object().ok_or_else(|| de::Error::custom("expected an object"))?;
                        Ok(#name::#variant_ident {
                            #( #field_names: serde_json::from_value(map.get(stringify!(#field_names)).ok_or_else(|| de::Error::missing_field(stringify!(#field_names)))?.clone()).map_err(de::Error::custom)? ),*
                        })
                    }
                }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                let field = &fields.unnamed[0];
                // TODO(tizoc): only for polymorphic variants tuples should be spliced
                // Check if the unnamed field is a tuple type and splice it
                if let syn::Type::Tuple(ref tuple) = field.ty {
                    let tuple_len = tuple.elems.len();
                    let tuple_pattern: Vec<_> = (0..tuple_len)
                        .map(|i| syn::Ident::new(&format!("elem{}", i), proc_macro2::Span::call_site()))
                        .collect();

                    let deserialize_elements = tuple_pattern.iter().enumerate().map(|(i, elem)| {
                        quote! {
                            let #elem = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(#i + 1, &self))?;
                        }
                    });

                    quote! {
                        #variant_name => {
                            #( #deserialize_elements )*
                            Ok(#name::#variant_ident(( #( #tuple_pattern ),* )))
                        }
                    }
                } else {
                    quote! {
                        #variant_name => {
                            let value = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
                            Ok(#name::#variant_ident(value))
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #variant_name => Ok(#name::#variant_ident),
                }
            }
            _ => panic!("SerdeYojsonEnum only supports unit, single-value tuple, and struct-like variants"),
        }
    });

    let binary_variant_deserialize_matches = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        match &variant.fields {
            Fields::Named(ref fields) => {
                let field_names: Vec<_> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                quote! {
                    #binary_enum_name::#variant_ident { #( #field_names ),* } => {
                        Ok(#name::#variant_ident { #( #field_names ),* })
                    }
                }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                quote! {
                    #binary_enum_name::#variant_ident(value) => {
                        Ok(#name::#variant_ident(value))
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    #binary_enum_name::#variant_ident => {
                        Ok(#name::#variant_ident)
                    }
                }
            }
            _ => panic!("Unsupported variant type!"),
        }
    });

    let expanded = quote! {
        #binary_enum_definition

        impl<'de> serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                use serde::ser::{SerializeTuple, SerializeTupleVariant, SerializeStructVariant};
                use serde::Serialize;
                match self {
                    #( #variant_matches )*
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    struct YojsonEnumVisitor;

                    impl<'de> serde::de::Visitor<'de> for YojsonEnumVisitor {
                        type Value = #name;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str("a tuple representing an enum variant")
                        }

                        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                        where
                            A: serde::de::SeqAccess<'de>,
                        {
                            use serde::de;
                            let variant: String = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
                            match variant.as_str() {
                                #( #variant_deserialize_matches )*
                                _ => Err(de::Error::unknown_variant(&variant, &[])),
                            }
                        }
                    }

                    deserializer.deserialize_tuple(2, YojsonEnumVisitor)
                } else {
                    use serde::Deserialize;
                    // Use the automatically derived Deserialize implementation for the binary representation
                    let binary_version = #binary_enum_name::deserialize(deserializer)?;
                    match binary_version {
                        #( #binary_variant_deserialize_matches )*
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn to_snake_case(input: &str) -> String {
    let mut snake_case = String::new();
    let mut chars = input.chars().peekable();

    if let Some(first_char) = chars.next() {
        snake_case.push(first_char); // Retain the first char in uppercase
    }

    for ch in chars {
        if ch.is_uppercase() {
            snake_case.push('_');
            snake_case.push(ch.to_ascii_lowercase());
        } else {
            snake_case.push(ch);
        }
    }

    snake_case
}
