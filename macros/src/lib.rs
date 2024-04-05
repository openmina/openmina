#[doc = include_str!("action_event.md")]
#[proc_macro_derive(ActionEvent, attributes(action_event))]
pub fn action_event(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    match action_event::expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

mod action_event;
