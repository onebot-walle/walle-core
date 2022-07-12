use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Lit, Meta, Result};

use super::{snake_case, try_from_idents};

pub(crate) fn action_internal(
    attr: &Attribute,
    input: &DeriveInput,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    let name = &input.ident;
    let s = match attr.parse_meta()? {
        Meta::Path(_) => snake_case(name.to_string()),
        Meta::NameValue(v) => match v.lit {
            Lit::Str(s) => s.value(),
            _ => return Err(Error::new(Span::call_site(), "expect string for action")),
        },
        _ => return Err(Error::new(Span::call_site(), "expect NameValue for action")),
    };
    match &input.data {
        Data::Struct(data) => {
            let idents = try_from_idents(&data.fields, quote!(a.params))?;
            Ok(quote!(
                impl #span::action::ActionDeclare for #name {
                    fn action() -> &'static str {
                        #s
                    }
                }

                impl TryFrom<&mut #span::action::Action> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(a: &mut #span::action::Action) -> Result<Self, Self::Error> {
                        use #span::util::value::ExtendedMapExt;
                        if a.action.as_str() != #s {
                            Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                a.action.clone(),
                            ))
                        } else {
                            Ok(Self #idents)
                        }
                    }
                }
            ))
        }
        Data::Enum(data) => {
            let mut vars = vec![];
            for var in &data.variants {
                // todo attr
                let ident = &var.ident;
                let s = snake_case(ident.to_string());
                let idents = try_from_idents(&var.fields, quote!(a))?;
                vars.push(quote!(
                    #s => Ok(Self::#ident #idents)
                ));
            }
            Ok(quote!(
                impl TryFrom<&mut #span::action::Action> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(a: &mut #span::action::Action) -> Result<Self, Self::Error> {
                        use #span::util::value::ExtendedMapExt;
                        match a.action.as_str() {
                            #(#vars,)*
                            _ => Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                a.action.clone(),
                            ))
                        }
                    }
                }
            ))
        }
        Data::Union(_) => Err(Error::new(Span::call_site(), "union not supported")),
    }
}
