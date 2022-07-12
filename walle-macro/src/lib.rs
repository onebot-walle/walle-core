use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, Type};

mod action;
mod event;

#[proc_macro_derive(OneBot, attributes(event, action, value))]
pub fn onebot(token: TokenStream) -> TokenStream {
    onebot_internal(token, quote!(walle_core))
}

#[proc_macro_derive(_OneBot, attributes(event, action, value))]
pub fn _onebot(token: TokenStream) -> TokenStream {
    onebot_internal(token, quote!(crate))
}

fn onebot_internal(token: TokenStream, span: TokenStream2) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    let mut stream = quote!();
    for attr in &input.attrs {
        if attr.path.is_ident("event") {
            stream.extend(flatten(event::event_internal(&attr, &input, &span)));
        } else if attr.path.is_ident("action") {
            stream.extend(flatten(action::action_internal(attr, &input, &span)));
        } else if attr.path.is_ident("value") {
            stream.extend(flatten(value_internal(&input, &span)));
        }
    }
    stream.into()
}

fn flatten(input: Result<TokenStream2>) -> TokenStream2 {
    match input {
        Ok(stream) => stream,
        Err(e) => e.into_compile_error(),
    }
}

fn value_internal(input: &DeriveInput, span: &TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    if let Data::Struct(data) = &input.data {
        let idents = try_from_idents(&data.fields, quote!(map))?;
        Ok(quote!(
            impl TryFrom<&mut #span::util::value::ExtendedMap> for #name {
                type Error = #span::error::WalleError;
                fn try_from(map: &mut #span::util::value::ExtendedMap) -> Result<Self, Self::Error> {
                    Ok(Self #idents )
                }
            }
            impl TryFrom<#span::util::value::ExtendedMap> for #name {
                type Error = #span::error::WalleError;
                fn try_from(mut map: #span::util::value::ExtendedMap) -> Result<Self, Self::Error> {
                    Self::try_from(&mut map)
                }
            }
            impl TryFrom<#span::util::value::ExtendedValue> for #name {
                type Error = #span::error::WalleError;
                fn try_from(v: #span::util::value::ExtendedValue) -> Result<Self, Self::Error> {
                    if let #span::util::value::ExtendedValue::Map(mut map) = v {
                        Self::try_from(&mut map)
                    } else {
                        Err(#span::error::WalleError::ValueTypeNotMatch(
                            "map".to_string(),
                            format!("{:?}", v),
                        ))
                    }
                }
            }
        ))
    } else {
        Err(Error::new(Span::call_site(), "value only support struct"))
    }
}

fn try_from_idents(fields: &Fields, head: TokenStream2) -> Result<TokenStream2> {
    if let Fields::Named(v) = &fields {
        let mut out = vec![];
        for field in &v.named {
            let ident = field.ident.clone().unwrap();
            let mut s = ident.to_string();
            if &s == "ty" {
                s = "type".to_string();
            }
            if let Type::Path(p) = &field.ty {
                if p.path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .to_string()
                    .starts_with("Option")
                {
                    out.push(quote!(
                        #ident: #head.try_remove_downcast(#s)?
                    ));
                    continue;
                }
            }
            out.push(quote!(
                #ident: #head.remove_downcast(#s)?
            ));
        }
        Ok(quote!({#(#out),*}))
    } else if let Fields::Unnamed(v) = &fields {
        let mut out = vec![];
        for field in &v.unnamed {
            let ty = &field.ty;
            out.push(quote!(
                #ty::try_from(#head)?
            ));
        }
        Ok(quote!((#(#out),*)))
    } else {
        Err(Error::new(Span::call_site(), "expect named struct"))
    }
}

#[proc_macro_derive(PushToMap)]
pub fn push_to_map(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(push_to_map_internal(input, quote!(walle_core))).into()
}

#[proc_macro_derive(_PushToMap)]
pub fn _push_to_map(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(push_to_map_internal(input, quote!(crate))).into()
}

fn push_to_map_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    let idents = push_idents(&input)?;
    Ok(quote!(
        impl #span::util::value::PushToExtendedMap for #name {
            fn push_to(self, map: &mut #span ::util::value::ExtendedMap) {
                #(#idents)*
            }
        }

        impl From<#name> for #span::util::value::ExtendedMap {
            fn from(i: #name) -> Self {
                use #span ::util::value::PushToExtendedMap;
                let mut map = Self::default();
                i.push_to(&mut map);
                map
            }
        }

        impl From<#name> for #span::util::value::ExtendedValue {
            fn from(i: #name) -> Self {
                #span::util::value::ExtendedValue::Map(i.into())
            }
        }
    ))
}

fn push_idents(input: &DeriveInput) -> Result<Vec<TokenStream2>> {
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(v) = &data.fields {
            let mut out = vec![];
            for field in &v.named {
                let i = field.ident.clone().unwrap();
                let mut s = i.to_string();
                if &s == "ty" {
                    s = "type".to_string();
                }
                out.push(quote!(
                    map.insert(#s.to_string(), self.#i.into());
                ));
            }
            Ok(out)
        } else {
            Err(Error::new(Span::call_site(), "expect named struct"))
        }
    } else {
        Err(Error::new(Span::call_site(), "expect struct"))
    }
}

fn snake_case(s: String) -> String {
    let mut out = String::default();
    let mut chars = s.chars();
    out.push(chars.next().unwrap().to_ascii_lowercase());
    while let Some(c) = chars.next() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
