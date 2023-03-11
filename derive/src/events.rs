use crate::{err, Derivable, DeriveParams};
use proc_macro2::{Ident, TokenStream};
use quote::quote_spanned;
use syn::{Data, DeriveInput, Generics, Result};

pub struct Events;

impl Derivable for Events {
    type Params = EventsParam;

    fn derive(params: EventsParam) -> Result<TokenStream> {
        let enum_ident = params.name;
        let span = enum_ident.span();

        let variants = params.variants.iter().map(|(variant_name, empty)| {
            let span = variant_name.span();
            if *empty {
                quote_spanned!(span => RawEventType::#variant_name => Self::#variant_name,)
            } else {
                quote_spanned!(span => RawEventType::#variant_name => Self::#variant_name(parse_event(raw.params).with_raw(raw)?),)
            }
        });

        let (impl_generics, ty_generics, where_clause) = params.generics.split_for_impl();

        Ok(
            quote_spanned!(span => impl #impl_generics #enum_ident #ty_generics #where_clause {
                pub fn parse(raw: &RawEvent<'a>) -> Result<Self, GameEventError> {
                    Ok(match raw.ty {
                        #(#variants)*
                        _ => {
                            todo!("{:?} not parsed yet", raw.ty);
                        }
                    })
                }
            }),
        )
    }
}

#[derive(Debug)]
pub struct EventsParam {
    name: Ident,
    generics: Generics,
    variants: Vec<(Ident, bool)>,
}

impl DeriveParams for EventsParam {
    fn parse(input: &DeriveInput) -> Result<EventsParam> {
        let Data::Enum(data) = &input.data else {
            return err("only supported on enums", input);
        };
        let name = input.ident.clone();
        let generics = input.generics.clone();

        let variants = data
            .variants
            .iter()
            .map(|variant| (variant.ident.clone(), variant.fields.is_empty()))
            .collect();

        Ok(EventsParam {
            name,
            generics,
            variants,
        })
    }
}
