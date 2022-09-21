use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, Type};

mod action_segment;
mod event;
mod value;

#[proc_macro_derive(OneBot, attributes(event, action, value, segment))]
pub fn onebot(token: TokenStream) -> TokenStream {
    onebot_internal(token, quote!(walle_core))
}

#[proc_macro_derive(_OneBot, attributes(event, action, value, segment))]
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
            stream.extend(flatten(action_segment::internal(attr, &input, &span, true)));
        } else if attr.path.is_ident("value") {
            stream.extend(flatten(value_internal(&input, &span)));
        } else if attr.path.is_ident("segment") {
            stream.extend(flatten(action_segment::internal(
                attr, &input, &span, false,
            )));
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
        let idents = try_from_idents(&data.fields, quote!(map), false)?;
        Ok(quote!(
            impl TryFrom<&mut #span::util::value::ValueMap> for #name {
                type Error = #span::error::WalleError;
                fn try_from(map: &mut #span::util::value::ValueMap) -> Result<Self, Self::Error> {
                    use #span::util::value::ValueMapExt;
                    Ok(Self #idents )
                }
            }
            impl TryFrom<#span::util::value::ValueMap> for #name {
                type Error = #span::error::WalleError;
                fn try_from(mut map: #span::util::value::ValueMap) -> Result<Self, Self::Error> {
                    Self::try_from(&mut map)
                }
            }
            impl TryFrom<#span::util::value::Value> for #name {
                type Error = #span::error::WalleError;
                fn try_from(v: #span::util::value::Value) -> Result<Self, Self::Error> {
                    if let #span::util::value::Value::Map(mut map) = v {
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

fn try_from_idents(fields: &Fields, head: TokenStream2, sub: bool) -> Result<TokenStream2> {
    match &fields {
        Fields::Named(v) => {
            let mut out = vec![];
            for field in &v.named {
                let ident = field.ident.clone().unwrap();
                let mut s = ident.to_string();
                escape(&mut s);
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
        }
        Fields::Unnamed(v) => {
            let mut out = vec![];
            for field in &v.unnamed {
                let ty = &field.ty;
                if sub {
                    out.push(quote!(
                        #ty::try_from(&mut #head)?
                    ));
                } else {
                    out.push(quote!(
                        #ty::try_from(#head)?
                    ));
                }
            }
            Ok(quote!((#(#out),*)))
        }
        Fields::Unit => Ok(quote!()),
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

fn escape(s: &mut String) {
    match s.as_str() {
        "ty" => *s = "type".to_string(),
        "implt" => *s = "impl".to_string(),
        "selft" => *s = "self".to_string(),
        _ => {}
    }
}

fn error<T: std::fmt::Display>(msg: T) -> Error {
    Error::new(Span::call_site(), msg)
}

macro_rules! ob {
    ($t: ident $(: $($attr: ident),+)? => $f: ident, $fin: ident, $span: ident) => {
        #[proc_macro_derive($t $(,attributes($($attr),+))?)]
        pub fn $f(token: TokenStream) -> TokenStream {
            let input = syn::parse_macro_input!(token as DeriveInput);
            flatten($fin(input, quote!($span))).into()
        }
    };
}

use value::try_from_value_internal;

ob!(TryFromValue => try_from_value, try_from_value_internal, walle_core);
ob!(_TryFromValue => _try_from_value, try_from_value_internal, crate);

use value::push_to_value_map_internal;

ob!(PushToValueMap => push_to_value_map, push_to_value_map_internal, walle_core);
ob!(_PushToValueMap => _push_to_value_map, push_to_value_map_internal, crate);

use event::to_event_internal;

ob!(ToEvent: event => to_event, to_event_internal, walle_core);
ob!(_ToEvent: event => _to_event, to_event_internal, crate);

fn fields_from_map(fields: &Fields) -> TokenStream2 {
    match fields {
        Fields::Named(named) => {
            let v = named
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.clone().unwrap();
                    let mut s = field_name.to_string();
                    escape(&mut s);
                    if let Type::Path(ref p) = f.ty {
                        if p.path
                            .segments
                            .first()
                            .unwrap()
                            .ident
                            .to_string()
                            .starts_with("Option")
                        {
                            return quote!(#field_name: map.try_remove_downcast(#s)?);
                        }
                    }
                    quote!(#field_name: map.remove_downcast(#s)?)
                })
                .collect::<Vec<_>>();
            quote!({#(#v),*})
        }
        Fields::Unnamed(unamed) => {
            let v = unamed
                .unnamed
                .iter()
                .map(|field| {
                    let t = &field.ty;
                    quote!(#t::tryfrom(map)?)
                })
                .collect::<Vec<_>>();
            quote!((#(#v),*))
        }
        Fields::Unit => quote!(),
    }
}
