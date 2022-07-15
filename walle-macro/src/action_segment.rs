use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Lit, Meta, Result};

use super::{snake_case, try_from_idents};

pub(crate) fn internal(
    attr: &Attribute,
    input: &DeriveInput,
    span: &TokenStream2,
    action: bool,
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
    let (declare, fn_name, from_ty, extra) = if action {
        (
            quote!(action::ActionDeclare),
            quote!(action),
            quote!(action::Action),
            quote!(params),
        )
    } else {
        (
            quote!(message::SegmentDeclare),
            quote!(ty),
            quote!(message::MessageSegment),
            quote!(data),
        )
    };
    match &input.data {
        Data::Struct(data) => {
            let idents = try_from_idents(
                &data.fields,
                if action {
                    quote!(v.params)
                } else {
                    quote!(v.data)
                },
            )?;
            Ok(quote!(
                impl #span::#declare for #name {
                    fn #fn_name() -> &'static str {
                        #s
                    }
                }

                impl TryFrom<&mut #span::#from_ty> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(v: &mut #span::#from_ty) -> Result<Self, Self::Error> {
                        use #span::util::value::ExtendedMapExt;
                        if v.#fn_name.as_str() != #s {
                            Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                v.#fn_name.clone(),
                            ))
                        } else {
                            Ok(Self #idents)
                        }
                    }
                }

                impl TryFrom<#span::#from_ty> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(mut v: #span::#from_ty) -> Result<Self, Self::Error> {
                        use #span::util::value::ExtendedMapExt;
                        if v.#fn_name.as_str() != #s {
                            Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                v.#fn_name.clone(),
                            ))
                        } else {
                            Ok(Self #idents)
                        }
                    }
                }

                impl From<#name> for #span::#from_ty {
                    fn from(v: #name) -> Self {
                        Self {
                            #fn_name: #s.to_string(),
                            #extra: v.into(),
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
                let idents = try_from_idents(&var.fields, quote!(v))?;
                vars.push(quote!(
                    #s => Ok(Self::#ident #idents)
                ));
            }
            Ok(quote!(
                impl TryFrom<&mut #span::#from_ty> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(v: &mut #span::#from_ty) -> Result<Self, Self::Error> {
                        use #span::util::value::ExtendedMapExt;
                        match v.#fn_name.as_str() {
                            #(#vars,)*
                            _ => Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                v.#fn_name.clone(),
                            ))
                        }
                    }
                }

                impl TryFrom<#span::#from_ty> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(mut v: #span::#from_ty) -> Result<Self, Self::Error> {
                        Self::try_from(&mut v)
                    }
                }
            ))
        }
        Data::Union(_) => Err(Error::new(Span::call_site(), "union not supported")),
    }
}
