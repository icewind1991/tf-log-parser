use crate::{Derivable, DeriveParams};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Data, DeriveInput, Error, Field, Fields, Generics, Lifetime, Result, Type, TypePath};
use syn_util::{contains_attribute, get_attribute_value};

macro_rules! bail {
    ($msg:expr $(,)?) => {
        return Err(Error::new(Span::call_site(), &$msg[..]))
    };

    ( $msg:expr => $span_to_blame:expr $(,)? ) => {
        return Err(Error::new_spanned(&$span_to_blame, $msg))
    };
}

pub struct Event;

impl Derivable for Event {
    type Params = EventParams;

    fn derive(params: EventParams) -> Result<TokenStream> {
        let struct_ident = params.name;
        let span = struct_ident.span();
        let required_params = params.params.iter().filter(|param| !param.optional);
        let optional_params = params.params.iter().filter(|param| param.optional);
        let has_optional = params.params.iter().any(|param| param.optional);

        let required_fields = required_params
            .map(|param| {
                let field_name = &param.field_name;

                let parser = match (&param.param_name, param.quoted) {
                    (Some(param_name), true) => {
                        quote_spanned!(field_name.span() => param_parse_with(#param_name, quoted(parse_field)))
                    }
                    (Some(param_name), false) => {
                        quote_spanned!(field_name.span() => param_parse_with(#param_name, parse_field))
                    }
                    (None, true) => {
                        quote_spanned!(field_name.span() => quoted(parse_field))
                    }
                    (None, false) => {
                        quote_spanned!(field_name.span() => parse_field)
                    }
                };

                let skip_after = param.skip_after as usize;
                let after = if skip_after > 0 {
                    quote_spanned!(field_name.span() => let input = &input[#skip_after..];)
                } else {
                    quote!()
                };

                Ok(quote_spanned!(field_name.span() =>
                    #[allow(unused_variables)]
                    let (input, #field_name) = #parser(input)?;
                    #after
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let initiators = params.params.iter().map(|param| {
            let field_name = &param.field_name;

            if param.optional {
                quote_spanned!(field_name.span() => #field_name: Default::default())
            } else {
                quote_spanned!(field_name.span() => #field_name)
            }
        });
        let initiator = quote!(
            #[allow(unused_mut)]
            let mut event = #struct_ident {
                #(#initiators),*
            };
        );
        let update = if has_optional {
            let matches = optional_params
                .map(|param| {
                    let field_name = &param.field_name;
                    let Some(param_name) = param.param_name.as_deref() else {
                        bail!("optional fields can't be unnamed" => param.field_name)
                    };

                    let parser = if param.quoted {
                        quote_spanned!(field_name.span() => quoted(parse_field))
                    } else {
                        quote_spanned!(field_name.span() => parse_field)
                    };

                    Ok(quote_spanned!(
                        field_name.span() => #param_name => event.#field_name = #parser(value)?.1
                    ))
                })
                .collect::<Result<Vec<_>>>()?;

            quote_spanned!(span => for (key, value) in ParamIter::new(input) {
                match key {
                    #(#matches,)*
                    _ => {}
                }
            })
        } else {
            quote!()
        };

        let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();

        let lifetime = params.lifetime;

        Ok(
            quote_spanned!(span => impl #impl_generics Event<#lifetime> for #struct_ident #ty_generics #where_clause {
                    fn parse(input: & #lifetime str) -> IResult<Self> {
                        #(#required_fields)*

                        #initiator

                        #update

                        Ok(("", event))
                    }
                }
            ),
        )
    }
}

#[derive(Debug)]
pub struct EventParams {
    name: Ident,
    lifetime: Lifetime,
    generics: Generics,
    params: Vec<EventParam>,
}

impl DeriveParams for EventParams {
    fn parse(input: &DeriveInput) -> Result<EventParams> {
        let Data::Struct(data) = &input.data else {
            bail!("only supported on structs" => input)
        };
        let Fields::Named(fields) = &data.fields else {
            bail!("only supported with named fields" => input)
        };
        let name = input.ident.clone();
        let generics = input.generics.clone();
        let params = fields
            .named
            .iter()
            .map(EventParam::parse)
            .collect::<Result<Vec<_>>>()?;

        let mut last_optional = false;
        for param in params.iter() {
            if last_optional > param.optional {
                bail!("optional fields are required to be at the end" => param.field_name)
            }
            last_optional = param.optional;
        }

        let lifetime = if let Some(lifetime) =
            get_attribute_value::<String>(&input.attrs, &["event", "lifetime"])
        {
            Lifetime::new(&lifetime, name.span())
        } else {
            let mut lifetimes = input.generics.lifetimes();
            let lifetime = lifetimes
                .next()
                .cloned()
                .map(|lifetime| lifetime.lifetime)
                .unwrap_or_else(|| Lifetime::new("'_", name.span()));
            if lifetimes.next().is_some() {
                bail!("For structs with more than one lifetime, manually specifiying the lifetime is required" => name);
            }
            lifetime
        };

        Ok(EventParams {
            name,
            lifetime,
            generics,
            params,
        })
    }
}

#[derive(Debug)]
pub struct EventParam {
    field_name: Ident,
    param_name: Option<String>,
    optional: bool,
    skip_after: u64,
    quoted: bool,
}

impl EventParam {
    pub fn parse(input: &Field) -> Result<EventParam> {
        let field_name = input.ident.clone().expect("no name on named fields");
        let param_name = if contains_attribute(&input.attrs, &["event", "unnamed"]) {
            None
        } else {
            Some(
                get_attribute_value(&input.attrs, &["event", "name"])
                    .unwrap_or_else(|| field_name.to_string()),
            )
        };
        let is_option = match &input.ty {
            Type::Path(TypePath { path, .. }) => {
                path.segments
                    .first()
                    .map(|segment| segment.ident.to_string())
                    .as_deref()
                    == Some("Option")
            }
            _ => false,
        };
        let optional = is_option || contains_attribute(&input.attrs, &["event", "default"]);
        let skip_after =
            get_attribute_value(&input.attrs, &["event", "skip_after"]).unwrap_or_default();

        if optional && skip_after > 0 {
            bail!("skip_after can't be used with optional fields" => input);
        }
        let quoted = contains_attribute(&input.attrs, &["event", "quoted"]);

        Ok(EventParam {
            field_name,
            param_name,
            optional,
            skip_after,
            quoted,
        })
    }
}
