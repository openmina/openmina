use std::convert::TryInto;

use proc_macro2::*;
use quote::*;
use syn::*;

#[derive(Clone, Debug)]
enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl TryFrom<Expr> for Level {
    type Error = Error;

    fn try_from(value: Expr) -> Result<Level> {
        let Expr::Path(ExprPath { path, .. }) = value else {
            return Err(Error::new_spanned(value, "ident is expected"));
        };
        let level = match path.require_ident()?.to_string().to_lowercase().as_str() {
            "error" => Level::Error,
            "warn" => Level::Warn,
            "info" => Level::Info,
            "debug" => Level::Debug,
            "trace" => Level::Trace,
            _ => return Err(Error::new_spanned(path, "incorrect value")),
        };
        Ok(level)
    }
}

impl ToTokens for Level {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = match self {
            Level::Error => format_ident!("action_error"),
            Level::Warn => format_ident!("action_warn"),
            Level::Info => format_ident!("action_info"),
            Level::Debug => format_ident!("action_debug"),
            Level::Trace => format_ident!("action_trace"),
        };
        tokens.extend(quote!(#ident));
    }
}

#[derive(Clone, Debug)]
enum FieldsSpec {
    /// List of expressions for fields to be added to the tracing, with
    /// optional ident for filtering.
    Some(Vec<(Option<Ident>, TokenStream)>),
}

#[derive(Clone, Debug, Default)]
struct ActionEventAttrs {
    level: Option<Level>,
    fields: Option<FieldsSpec>,
}

pub fn expand(input: DeriveInput) -> Result<TokenStream> {
    let Data::Enum(enum_data) = &input.data else {
        return Err(Error::new_spanned(input, "should be enum"));
    };
    let type_name = &input.ident;
    let trait_name = quote!(openmina_core::ActionEvent); // TODO
    let input_attrs = action_event_attrs(&input.attrs)?;
    let variants = enum_data
        .variants
        .iter()
        .map(|v| {
            let variant_name = &v.ident;
            let mut args = vec![quote!(context)];
            let variant_attrs = action_event_attrs(&v.attrs)?;
            match &v.fields {
                Fields::Unnamed(fields) => {
                    if fields.unnamed.len() != 1 {
                        return Err(Error::new_spanned(
                            fields,
                            "only single-item variant supported",
                        ));
                    }
                    if fields.unnamed.len() != 1 {
                        return Err(Error::new_spanned(
                            fields,
                            "only single-item variant supported",
                        ));
                    }
                    Ok(quote! {
                        #type_name :: #variant_name (action) => action.action_event(#(#args),*),
                    })
                }
                Fields::Named(fields_named) => {
                    let field_names = fields_named.named.iter().map(|named| &named.ident);
                    args.extend(summary_field(&v.attrs)?);
                    args.extend(fields(&variant_attrs.fields, &input_attrs.fields, fields_named)?);
                    let level = level(&variant_attrs.level, &v.ident, &input_attrs.level);
                    Ok(quote! {
                        #type_name :: #variant_name { #(#field_names),* } => openmina_core::#level!(#(#args),*),
                    })
                }
                Fields::Unit => {
                    args.extend(summary_field(&v.attrs)?);
                    let level = level(&variant_attrs.level, &v.ident, &input_attrs.level);
                    Ok(quote! {
                        #type_name :: #variant_name => openmina_core::#level!(#(#args),*),
                    })
                }
            }
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl #trait_name for #type_name {
            fn action_event<T>(&self, context: &T)
            where T: openmina_core::log::EventContext,
            {
                #[allow(unused_variables)]
                match self {
                    #(#variants)*
                }
            }
        }
    })
}

fn level(variant_level: &Option<Level>, variant_name: &Ident, enum_level: &Option<Level>) -> Level {
    variant_level
        .as_ref()
        .cloned()
        .or_else(|| {
            let s = variant_name.to_string();
            (s.ends_with("Error") || s.ends_with("Warn")).then_some(Level::Warn)
        })
        .or_else(|| enum_level.as_ref().cloned())
        .unwrap_or(Level::Debug)
}

fn fields(
    variant_fields: &Option<FieldsSpec>,
    enum_fields: &Option<FieldsSpec>,
    fields: &FieldsNamed,
) -> Result<Vec<TokenStream>> {
    variant_fields
        .as_ref()
        .or(enum_fields.as_ref())
        .map_or_else(|| Ok(Vec::new()), |f| filter_fields(f, fields))
}

fn filter_fields(field_spec: &FieldsSpec, fields: &FieldsNamed) -> Result<Vec<TokenStream>> {
    match field_spec {
        FieldsSpec::Some(f) => f
            .iter()
            .filter(|(name, _)| {
                name.as_ref().map_or(true, |name| {
                    fields.named.iter().any(|n| Some(name) == n.ident.as_ref())
                })
            })
            .map(|(_, expr)| Ok(expr.clone()))
            .collect(),
    }
}

fn action_event_attrs(attrs: &Vec<Attribute>) -> Result<ActionEventAttrs> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("action_event"))
        .try_fold(ActionEventAttrs::default(), |mut attrs, attr| {
            let nested =
                attr.parse_args_with(punctuated::Punctuated::<Meta, Token![,]>::parse_terminated)?;
            nested.into_iter().try_for_each(|meta| {
                match meta {
                    // #[level = ...]
                    Meta::NameValue(name_value) if name_value.path.is_ident("level") => {
                        let _ = attrs.level.insert(name_value.value.try_into()?);
                    }
                    // #[fields(...)]
                    Meta::List(list) if list.path.is_ident("fields") => {
                        let nested = list.parse_args_with(
                            punctuated::Punctuated::<Meta, Token![,]>::parse_terminated,
                        )?;
                        let fields = nested
                            .iter()
                            .map(|meta| {
                                match meta {
                                    // field
                                    Meta::Path(path) => {
                                        let ident = path.require_ident()?;
                                        Ok((Some(ident.clone()), quote!(#ident = #ident)))
                                    }
                                    // field = expr
                                    Meta::NameValue(name_value) => {
                                        let event_field = name_value.path.require_ident()?;
                                        let expr = &name_value.value;
                                        let maybe_field = get_field_name(expr);
                                        Ok((maybe_field.cloned(), quote!(#event_field = #expr)))
                                    }
                                    // debug(field)
                                    // display(field)
                                    Meta::List(list)
                                        if list.path.is_ident("debug")
                                            || list.path.is_ident("display") =>
                                    {
                                        let conv = list.path.require_ident()?;
                                        let Expr::Path(field) = list.parse_args::<Expr>()? else {
                                            return Err(Error::new_spanned(
                                                list,
                                                "identifier is expected",
                                            ));
                                        };
                                        let field = field.path.require_ident()?;
                                        Ok((Some(field.clone()), quote!(#field = #conv(#field))))
                                    }
                                    _ => Err(Error::new_spanned(meta, "unrecognized repr")),
                                }
                            })
                            .collect::<Result<Vec<_>>>()?;
                        let _ = attrs.fields.insert(FieldsSpec::Some(fields));
                    }
                    _ => return Err(Error::new_spanned(meta, "unrecognized repr")),
                }
                Ok(())
            })?;
            Ok(attrs)
        })
}

fn get_field_name(expr: &Expr) -> Option<&Ident> {
    match expr {
        Expr::Path(path) => path.path.require_ident().ok(),
        Expr::Field(field) => get_field_name(&field.base),
        Expr::Reference(reference) => get_field_name(&reference.expr),
        Expr::Unary(ExprUnary { expr, .. }) => get_field_name(expr),
        Expr::Call(call) => match call.func.as_ref() {
            Expr::Path(path) if path.path.is_ident("display") || path.path.is_ident("debug") => {
                call.args.first().and_then(|arg| get_field_name(arg))
            }
            Expr::Field(field) => get_field_name(&field.base),
            _ => None,
        },
        _ => None,
    }
}

fn summary_field(attrs: &Vec<Attribute>) -> Result<Option<TokenStream>> {
    let Some(doc_attr) = attrs.iter().find(|attr| attr.path().is_ident("doc")) else {
        return Ok(None);
    };
    let name_value = doc_attr.meta.require_name_value()?;
    let Expr::Lit(ExprLit {
        lit: Lit::Str(lit), ..
    }) = &name_value.value
    else {
        return Ok(None);
    };
    let value = lit.value();
    let trimmed = value.trim();
    let stripped = trimmed.strip_suffix('.').unwrap_or(trimmed);
    Ok(Some(quote!(summary = #stripped)))
}

#[cfg(test)]
mod tests {
    use rust_format::{Formatter, RustFmt};

    fn test(input: &str, expected: &str) -> anyhow::Result<()> {
        let fmt = RustFmt::default();

        let expected = fmt.format_str(expected)?;
        let input = syn::parse_str::<syn::DeriveInput>(input)?;
        let output = super::expand(input)?;
        let output = fmt.format_tokens(output)?;
        assert_eq!(
            output, expected,
            "\n<<<<<<\n{}======\n{}>>>>>>",
            output, expected
        );
        Ok(())
    }

    #[test]
    fn test_delegate() -> anyhow::Result<()> {
        let input = r#"
#[derive(openmina_macros::ActionEvent)]
pub enum SuperAction {
    Sub1(SubAction1),
    Sub2(SubAction2),
}
"#;
        let expected = r#"
impl openmina_core::ActionEvent for SuperAction {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            SuperAction::Sub1(action) => action.action_event(context),
            SuperAction::Sub2(action) => action.action_event(context),
        }
    }
}
"#;
        test(input, expected)
    }

    #[test]
    fn test_unit() -> anyhow::Result<()> {
        let input = r#"
#[derive(openmina_macros::ActionEvent)]
pub enum Action {
    Unit,
    /// documentation
    UnitWithDoc,
    /// Multiline documentation.
    /// Another line.
    ///
    /// And another.
    UnitWithMultilineDoc,
}
"#;
        let expected = r#"
impl openmina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::Unit => openmina_core::action_debug!(context),
            Action::UnitWithDoc => openmina_core::action_debug!(context, summary = "documentation"),
            Action::UnitWithMultilineDoc => openmina_core::action_debug!(context, summary = "Multiline documentation"),
        }
    }
}
"#;
        test(input, expected)
    }

    #[test]
    fn test_level() -> anyhow::Result<()> {
        let input = r#"
#[derive(openmina_macros::ActionEvent)]
#[action_event(level = trace)]
pub enum Action {
    ActionDefaultLevel,
    #[action_event(level = warn)]
    ActionOverrideLevel,
    ActionWithError,
    ActionWithWarn,
}
"#;
        let expected = r#"
impl openmina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::ActionDefaultLevel => openmina_core::action_trace!(context),
            Action::ActionOverrideLevel => openmina_core::action_warn!(context),
            Action::ActionWithError => openmina_core::action_warn!(context),
            Action::ActionWithWarn => openmina_core::action_warn!(context),
        }
    }
}
"#;
        test(input, expected)
    }

    #[test]
    fn test_fields() -> anyhow::Result<()> {
        let input = r#"
#[derive(openmina_core::ActionEvent)]
pub enum Action {
    NoFields { f1: bool },
    #[action_event(fields(f1))]
    Field { f1: bool },
    #[action_event(fields(f = f1))]
    FieldWithName { f1: bool },
    #[action_event(fields(f = f.subfield))]
    FieldExpr { f: WithSubfield },
    #[action_event(fields(f = display(f.subfield)))]
    FieldDisplayExpr { f: WithSubfield },
    #[action_event(fields(debug(f1)))]
    DebugField { f1: bool },
    #[action_event(fields(display(f1)))]
    DisplayField { f1: bool },
}
"#;
        let expected = r#"
impl openmina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::NoFields { f1 } => openmina_core::action_debug!(context),
            Action::Field { f1 } => openmina_core::action_debug!(context, f1 = f1),
            Action::FieldWithName { f1 } => openmina_core::action_debug!(context, f = f1),
            Action::FieldExpr { f } => openmina_core::action_debug!(context, f = f.subfield),
            Action::FieldDisplayExpr { f } => openmina_core::action_debug!(context, f = display(f.subfield)),
            Action::DebugField { f1 } => openmina_core::action_debug!(context, f1 = debug(f1)),
            Action::DisplayField { f1 } => openmina_core::action_debug!(context, f1 = display(f1)),
        }
    }
}
"#;
        test(input, expected)
    }

    #[test]
    fn test_filtered_fields() -> anyhow::Result<()> {
        let input = r#"
#[derive(openmina_core::ActionEvent)]
#[action_event(fields(f1, f2 = f2.sub, f3 = display(f3.sub), f4 = foo()))]
pub enum Action {
    Unit,
    AllFields { f1: bool, f2: WithSub, f3: WithSub },
    OnlyF1 { f1: bool },
    WithF3 { f1: bool, f3: WithSub },
}
"#;
        let expected = r#"
impl openmina_core::ActionEvent for Action {
    fn action_event<T>(&self, context: &T)
    where
        T: openmina_core::log::EventContext,
    {
        #[allow(unused_variables)]
        match self {
            Action::Unit => openmina_core::action_debug!(context),
            Action::AllFields { f1, f2, f3 } => openmina_core::action_debug!(context, f1 = f1, f2 = f2.sub, f3 = display(f3.sub), f4 = foo()),
            Action::OnlyF1 { f1 } => openmina_core::action_debug!(context, f1 = f1, f4 = foo()),
            Action::WithF3 { f1, f3 } => openmina_core::action_debug!(context, f1 = f1, f3 = display(f3.sub), f4 = foo()),
        }
    }
}
"#;
        test(input, expected)
    }
}
