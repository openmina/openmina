#[doc = include_str!("action_event.md")]
#[proc_macro_derive(ActionEvent, attributes(action_event))]
pub fn action_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match action_event::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[doc = include_str!("serde_yojson_enum.md")]
#[proc_macro_derive(SerdeYojsonEnum)]
pub fn serde_yojson_enum_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    serde_yojson_enum::serde_yojson_enum_derive(input)
}

mod action_event;
mod serde_yojson_enum;
