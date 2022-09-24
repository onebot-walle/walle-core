use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Lit, Meta, NestedMeta, Result};

use crate::{error, fields_from_map, snake_case};

pub(crate) fn to_event_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    let name = input.ident;
    let (tys, _, ss) = attrs_parse(&name, &input.attrs, &span)?;

    match input.data {
        Data::Struct(_) | Data::Union(_) => Ok(quote!(
            #(impl #span::event::ToEvent<#tys> for #name {
                fn ty(&self) -> &'static str {
                    #ss
                }
            })*
        )),
        Data::Enum(ref data) => {
            let v = data
                .variants
                .iter()
                .map(|var| {
                    // todo attr
                    let id = var.ident.clone();
                    let s = snake_case(id.to_string());
                    match var.fields {
                        Fields::Named(_) => quote!(Self::#id {..} => #s),
                        Fields::Unnamed(_) => quote!(Self::#id (..) => #s),
                        Fields::Unit => quote!(Self::#id => #s),
                    }
                })
                .collect::<Vec<_>>();
            let v = quote!(#(#v),*);
            Ok(quote!(
                #(impl #span::event::ToEvent<#tys> for #name {
                    fn ty(&self) -> &'static str {
                        match self {
                            #v
                        }
                    }
                })*
            ))
        }
    }
}

pub(crate) fn try_from_event_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = input.ident;
    let (tys, tids, ss) = attrs_parse(&name, &input.attrs, &span)?;

    match input.data {
        Data::Union(_) => Ok(quote!(
            #(impl #span::event::TryFromEvent<#tys> for #name {
                fn try_from_event_mut(event: &mut #span::event::Event, implt: &str) -> #span::WalleResult<Self> {
                    if #tids == #ss {
                        Ok(Self)
                    } else {
                        Err(#span::WalleError::DeclareNotMatch(#ss, #tids.to_string()))
                    }
                }
            })*
        )),
        Data::Struct(data) => {
            let fs = fields_from_map(&data.fields);
            Ok(quote!(
                #(impl #span::event::TryFromEvent<#tys> for #name {
                    fn try_from_event_mut(event: &mut #span::event::Event, implt: &str) -> #span::WalleResult<Self> {
                        use #span::util::value::ValueMapExt;
                        if #tids == #ss {
                            let map = &mut event.extra;
                            Ok(Self #fs)
                        } else {
                            Err(#span::WalleError::DeclareNotMatch(#ss, #tids.to_string()))
                        }
                    }
                })*
            ))
        }
        Data::Enum(data) => {
            let mut ss = Vec::new();
            let arms = data
                .variants
                .into_iter()
                .map(|v| {
                    let vname = v.ident;
                    // todo attr
                    let s = snake_case(vname.to_string());
                    ss.push(s.clone());
                    let fs = fields_from_map(&v.fields);
                    quote!(#s => Ok(Self::#vname #fs))
                })
                .collect::<Vec<_>>();
            let arms = quote!(#(#arms,)*);
            let ss = ss.join("|");
            Ok(quote!(
                #(impl #span::event::TryFromEvent<#tys> for #name {
                    fn try_from_event_mut(event: &mut #span::event::Event, implt: &str) -> #span::WalleResult<Self> {
                        use #span::util::value::ValueMapExt;
                        let map = &mut event.extra;
                        match #tids {
                            #arms
                            _ => Err(#span::WalleError::DeclareNotMatch(#ss, #tids.to_string()))
                        }
                    }
                })*
            ))
        }
    }
}

fn parse_ty(ty: &str, span: &TokenStream2) -> Result<(TokenStream2, TokenStream2)> {
    match ty {
        "type" => Ok((quote!(#span::event::TypeLevel), quote!(event.ty.as_str()))),
        "detail_type" => Ok((
            quote!(#span::event::DetailTypeLevel),
            quote!(event.detail_type.as_str()),
        )),
        "sub_type" => Ok((
            quote!(#span::event::SubTypeLevel),
            quote!(event.sub_type.as_str()),
        )),
        "platform" => Ok((
            quote!(#span::event::PlatformLevel),
            quote!(event.selft().unwrap_or_default().platform.as_str()),
        )),
        "impl" => Ok((quote!(#span::event::ImplLevel), quote!(implt))),
        ty => Err(error(format!("unsupportted type {}", ty))),
    }
}

fn attrs_parse(
    name: &Ident,
    attrs: &Vec<Attribute>,
    span: &TokenStream2,
) -> Result<(Vec<TokenStream2>, Vec<TokenStream2>, Vec<String>)> {
    let mut vs = (Vec::new(), Vec::new(), Vec::new());
    for attr in attrs {
        if attr.path.is_ident("event") {
            meta_parse(name, attr.parse_meta()?, span, &mut vs)?;
        }
    }
    if vs.0.is_empty() {
        Err(error("miss attr event"))
    } else {
        Ok(vs)
    }
}

fn meta_parse(
    name: &Ident,
    meta: Meta,
    span: &TokenStream2,
    vs: &mut (Vec<TokenStream2>, Vec<TokenStream2>, Vec<String>),
) -> Result<()> {
    match meta {
        Meta::List(l) => {
            for nest in l.nested {
                match nest {
                    NestedMeta::Lit(_) => return Err(error("unexpect lit")),
                    NestedMeta::Meta(meta) => meta_parse(name, meta, span, vs)?,
                }
            }
        }
        Meta::Path(p) => {
            let (t0, t1) = parse_ty(&p.get_ident().unwrap().to_string(), span)?;
            vs.0.push(t0);
            vs.1.push(t1);
            vs.2.push(snake_case(name.to_string()));
        }
        Meta::NameValue(v) => {
            if let Lit::Str(s) = &v.lit {
                let (t0, t1) = parse_ty(&v.path.get_ident().unwrap().to_string(), span)?;
                vs.0.push(t0);
                vs.1.push(t1);
                vs.2.push(s.value());
            }
        }
    }
    Ok(())
}
