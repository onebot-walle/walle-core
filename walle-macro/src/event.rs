use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Lit, Meta, NestedMeta, Result,
};

use super::{snake_case, try_from_idents};

struct EventTypeInfo {
    pub name: Ident,
    pub ty: Option<String>,
    pub detail_type: Option<String>,
    pub sub_type: Option<String>,
    pub implt: Option<String>,
    pub platform: Option<String>,
}

impl EventTypeInfo {
    fn build_impl(&self, span: &TokenStream2) -> TokenStream2 {
        let name = &self.name;
        let mut token = quote!();
        if let Some(s) = &self.ty {
            token.extend(quote!(
                impl #span ::event::TypeDeclare for #name {
                    fn ty() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.detail_type {
            token.extend(quote!(
                impl #span ::event::DetailTypeDeclare for #name {
                    fn detail_type() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.sub_type {
            token.extend(quote!(
                impl #span ::event::SubTypeDeclare for #name {
                    fn sub_type() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.implt {
            token.extend(quote!(
                impl #span ::event::ImplDeclare for #name {
                    fn implt() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.platform {
            token.extend(quote!(
                impl #span ::event::PlatformDeclare for #name {
                    fn platform() -> &'static str {
                        #s
                    }
                }
            ));
        }
        token
    }

    fn build_check(&self, span: &TokenStream2) -> TokenStream2 {
        let mut token = quote!();
        if let Some(s) = &self.ty {
            token = quote!(
                #token
                if e.ty != #s {
                    return Err(#span ::error::WalleError::DeclareNotMatch(
                        #s,
                        e.ty.clone(),
                ))}
            )
        }
        if let Some(s) = &self.detail_type {
            token = quote!(
                #token
                if e.detail_type != #s {
                    return Err(#span ::error::WalleError::DeclareNotMatch(
                        #s,
                        e.detail_type.clone(),
                ))}
            )
        }
        if let Some(s) = &self.sub_type {
            token = quote!(
                #token
                if e.sub_type != #s {
                    return Err(#span ::error::WalleError::DeclareNotMatch(
                        #s,
                        e.sub_type.clone(),
                ))}
            )
        }
        if let Some(s) = &self.implt {
            token = quote!(
                #token
                if e.implt != #s {
                    return Err(#span ::error::WalleError::DeclareNotMatch(
                        #s,
                        e.implt.clone(),
                ))}
            )
        }
        if let Some(s) = &self.platform {
            token = quote!(
                #token
                if e.platform != #s {
                    return Err(#span ::error::WalleError::DeclareNotMatch(
                        #s,
                        e.platform.clone(),
                ))}
            )
        }
        token
    }
}

pub(crate) fn event_internal(
    attr: &Attribute,
    input: &DeriveInput,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    match &input.data {
        Data::Struct(data) => {
            let mut info = EventTypeInfo {
                name: input.ident.clone(),
                ty: None,
                detail_type: None,
                sub_type: None,
                implt: None,
                platform: None,
            };
            let mut event = event_declare(attr, &mut info, span)?;
            event.extend(event_struct_impl(data, &info, span)?);
            Ok(event)
        }
        Data::Enum(data) => {
            let mut ty = None;
            if let Meta::List(l) = attr.parse_meta()? {
                for nmeta in l.nested {
                    match nmeta {
                        NestedMeta::Meta(Meta::Path(p)) => {
                            match p.get_ident().unwrap().to_string().as_str() {
                                "type" => ty = Some(quote!(ty)),
                                "detail_type" => ty = Some(quote!(detail_type)),
                                "sub_type" => ty = Some(quote!(sub_type)),
                                "platform" => ty = Some(quote!(platform)),
                                "impl" => ty = Some(quote!(implt)),
                                _ => return Err(Error::new(Span::call_site(), "unkown type")),
                            }
                        }
                        _ => return Err(Error::new(Span::call_site(), "unsupport attributes")),
                    }
                }
            }
            if let Some(ty) = ty {
                event_enum_impl(&input.ident, &data, span, ty)
            } else {
                Err(Error::new(Span::call_site(), "need a type attribute"))
            }
        }
        _ => return Err(Error::new(Span::call_site(), "union not supported")),
    }
}

fn event_declare(
    attr: &Attribute,
    info: &mut EventTypeInfo,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    if let Meta::List(l) = attr.parse_meta()? {
        for nmeta in l.nested {
            match nmeta {
                NestedMeta::Meta(Meta::NameValue(v)) => {
                    if let Lit::Str(str) = v.lit {
                        match v.path.get_ident().unwrap().to_string().as_str() {
                            "type" => info.ty = Some(str.value()),
                            "detail_type" => info.detail_type = Some(str.value()),
                            "sub_type" => info.sub_type = Some(str.value()),
                            "platform" => info.platform = Some(str.value()),
                            "impl" => info.implt = Some(str.value()),
                            _ => return Err(Error::new(Span::call_site(), "unkown type")),
                        }
                    } else {
                        return Err(Error::new(Span::call_site(), "unsupport attributes"));
                    }
                }
                NestedMeta::Meta(Meta::Path(p)) => {
                    let s = snake_case(info.name.to_string());
                    match p.get_ident().unwrap().to_string().as_str() {
                        "type" => info.ty = Some(s),
                        "detail_type" => info.detail_type = Some(s),
                        "sub_type" => info.sub_type = Some(s),
                        "platform" => info.platform = Some(s),
                        "impl" => info.implt = Some(s),
                        _ => return Err(Error::new(Span::call_site(), "unkown type")),
                    }
                }
                _ => return Err(Error::new(Span::call_site(), "unsupport attributes")),
            }
        }
    }
    Ok(info.build_impl(span))
}

fn event_struct_impl(
    data: &DataStruct,
    info: &EventTypeInfo,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    let name = &info.name;
    let idents = try_from_idents(&data.fields, quote!(e.extra))?;
    let check = info.build_check(span);
    Ok(quote!(
        impl TryFrom<&mut #span ::event::Event> for #name {
            type Error = #span ::error::WalleError;
            fn try_from(e: &mut #span ::event::Event) -> Result<Self, Self::Error> {
                use #span ::util::value::ValueMapExt;
                #check
                Ok(Self #idents)
            }
        }
    ))
}

fn event_enum_impl(
    name: &Ident,
    data: &DataEnum,
    span: &TokenStream2,
    ty: TokenStream2,
) -> Result<TokenStream2> {
    let vars = data
        .variants
        .iter()
        .map(|v| -> Result<TokenStream2> {
            let ident = &v.ident;
            let s = snake_case(ident.to_string());
            let idents = try_from_idents(&v.fields, quote!(e))?;
            Ok(quote!(
                #s => Ok(Self::#ident #idents)
            ))
        })
        .collect::<Result<Vec<TokenStream2>>>()?;
    Ok(quote!(
        impl TryFrom<&mut #span::event::Event> for #name {
            type Error = #span::error::WalleError;
            fn try_from(e: &mut #span::event::Event) -> Result<Self, Self::Error> {
                use #span::util::value::ValueMapExt;
                match e.#ty.as_str() {
                    #(#vars,)*
                    _ => Err(#span::error::WalleError::DeclareNotMatch(
                        "event types",
                        e.#ty.clone(),
                    ))
                }
            }
        }
    ))
}
