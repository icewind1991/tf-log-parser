use crate::{err, Derivable, DeriveParams};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use structmeta::StructMeta;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DeriveInput, Field, Fields, Generics, Lifetime, LitBool, LitInt, LitStr,
    Result, Type, TypePath,
};

pub struct Event;

impl Derivable for Event {
    type Params = EventParams;

    fn derive(params: EventParams) -> Result<TokenStream> {
        let struct_ident = params.name;
        let span = struct_ident.span();
        let required_params = params.params.iter().filter(|param| !param.optional);
        let optional_params = params.params.iter().filter(|param| param.optional);
        let has_optional = params.params.iter().any(|param| param.optional);

        let single_unnamed_field =
            params.params.len() == 1 && params.params[0].param_name.is_none();

        let required_fields = required_params
            .map(|param| {
                let field_name = &param.field_name;
                let parser = param.parser();
                let after = param.skip_after();

                if single_unnamed_field {
                    Ok(quote_spanned!(param.span() =>
                        #[allow(unused_variables)]
                        let (input, #field_name) = match #parser(input) {
                            Ok(res) => res,
                            Err(Error::Incomplete) => ("", input.trim_matches('"')),
                            Err(e) => {
                                return Err(e);
                            }
                        };
                        #after
                    ))
                } else {
                    Ok(quote_spanned!(param.span() =>
                        #[allow(unused_variables)]
                        let (input, #field_name) = #parser(input)?;
                        #after
                    ))
                }
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
                        return err("optional fields can't be unnamed", &param.field_name);
                    };

                    Ok(quote_spanned!(
                        field_name.span() => #param_name => event.#field_name = parse_field(value)?
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
                    fn parse(input: & #lifetime str) -> Result<Self> {
                        #(#required_fields)*

                        #initiator

                        #update

                        Ok(event)
                    }
                }
            ),
        )
    }
}

pub struct EventParams {
    name: Ident,
    lifetime: Lifetime,
    generics: Generics,
    params: Vec<EventParam>,
}

impl DeriveParams for EventParams {
    fn parse(input: &DeriveInput) -> Result<EventParams> {
        let attrs: EventAttr = parse_attrs(&input.attrs);
        let Data::Struct(data) = &input.data else {
            return err("only supported on structs", input);
        };
        let Fields::Named(fields) = &data.fields else {
            return err("only supported with named fields", input);
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
                return err(
                    "optional fields are required to be at the end",
                    &param.field_name,
                );
            }
            last_optional = param.optional;
        }

        let lifetime = if let Some(lifetime) = attrs.lifetime {
            lifetime
        } else {
            let mut lifetimes = input.generics.lifetimes().cloned();
            let lifetime = lifetimes
                .next()
                .map(|lifetime| lifetime.lifetime)
                .unwrap_or_else(|| Lifetime::new("'_", name.span()));
            if lifetimes.next().is_some() {
                return err("For structs with more than one lifetime, manually specifiying the lifetime is required", name);
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

pub struct EventParam {
    span: Span,
    field_name: Ident,
    param_name: Option<String>,
    optional: bool,
    skip_after: u64,
    quoted: bool,
    subject: bool,
}

impl EventParam {
    pub fn parse(input: &Field) -> Result<EventParam> {
        let attrs: EventAttr = parse_attrs(&input.attrs);
        let field_name = input.ident.clone().expect("no name on named fields");
        let param_name = if attrs.unnamed {
            None
        } else {
            Some(
                attrs
                    .name
                    .map(|lit| lit.value())
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
        let optional = is_option || attrs.default;
        let skip_after = attrs
            .skip_after
            .and_then(|lit| lit.base10_parse().ok())
            .unwrap_or_default();

        if optional && skip_after > 0 {
            return err("skip_after can't be used with optional fields", input);
        }
        let quoted = attrs
            .quoted
            .map(|lit| lit.value)
            .unwrap_or(param_name.is_none());
        let subject = attrs.subject;

        Ok(EventParam {
            span: input.span(),
            field_name,
            param_name,
            optional,
            skip_after,
            quoted,
            subject,
        })
    }

    fn span(&self) -> Span {
        self.span
    }

    fn field_parser(&self) -> TokenStream {
        if self.subject {
            quote_spanned!(self.span() => subject_parser)
        } else if self.quoted {
            quote_spanned!(self.span() => quoted(parse_field))
        } else {
            quote_spanned!(self.span() => parse_field)
        }
    }

    fn parser(&self) -> TokenStream {
        let field_parser = self.field_parser();
        if let Some(param_name) = &self.param_name {
            quote_spanned!(self.span() => param_parse_with(#param_name, #field_parser))
        } else {
            quote_spanned!(self.span() => #field_parser)
        }
    }

    fn skip_after(&self) -> TokenStream {
        let skip_after = self.skip_after as usize;
        if skip_after > 0 {
            quote_spanned!(self.span() => let input = &input.get(#skip_after..).ok_or(Error::Incomplete)?;)
        } else {
            quote!()
        }
    }
}

fn parse_attrs(attrs: &[Attribute]) -> EventAttr {
    let mut result = EventAttr::default();
    for attr in attrs {
        if let Ok(parsed) = attr.parse_args() {
            result = result.merge(parsed);
        }
    }
    result
}

#[derive(Default, StructMeta)]
struct EventAttr {
    quoted: Option<LitBool>,
    subject: bool,
    default: bool,
    unnamed: bool,
    skip_after: Option<LitInt>,
    name: Option<LitStr>,
    lifetime: Option<Lifetime>,
}

impl EventAttr {
    fn merge(self, other: Self) -> Self {
        Self {
            quoted: self.quoted.or(other.quoted),
            subject: self.subject || other.subject,
            default: self.default || other.default,
            unnamed: self.unnamed || other.unnamed,
            skip_after: self.skip_after.or(other.skip_after),
            name: self.name.or(other.name),
            lifetime: self.lifetime.or(other.lifetime),
        }
    }
}
