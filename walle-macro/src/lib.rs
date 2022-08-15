use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, FieldsNamed, Result, Type};

mod action_segment;
mod event;

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
        let idents = try_from_idents(&data.fields, quote!(map))?;
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

fn try_from_idents(fields: &Fields, head: TokenStream2) -> Result<TokenStream2> {
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
                out.push(quote!(
                    #ty::try_from(#head)?
                ));
            }
            Ok(quote!((#(#out),*)))
        }
        Fields::Unit => Ok(quote!()),
    }
}

#[proc_macro_derive(PushToValueMap)]
pub fn push_to_map(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(push_to_map_internal(input, quote!(walle_core))).into()
}

#[proc_macro_derive(_PushToValueMap)]
pub fn _push_to_map(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(push_to_map_internal(input, quote!(crate))).into()
}

fn push_to_map_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    let idents = push_idents(&input, &span)?;
    Ok(quote!(
        impl #span::util::value::PushToValueMap for #name {
            fn push_to(self, map: &mut #span ::util::value::ValueMap) {
                #idents
            }
        }

        impl From<#name> for #span::util::value::ValueMap {
            fn from(i: #name) -> Self {
                use #span::util::value::PushToValueMap;
                let mut map = Self::default();
                i.push_to(&mut map);
                map
            }
        }

        impl From<#name> for #span::util::value::Value {
            fn from(i: #name) -> Self {
                #span::util::value::Value::Map(i.into())
            }
        }
    ))
}

fn push_idents(input: &DeriveInput, span: &TokenStream2) -> Result<TokenStream2> {
    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => named_push_idents(&named, quote!(self)),
            Fields::Unit => Ok(quote!()),
            Fields::Unnamed(unnamed) => {
                let mut i = 0;
                let mut vars = vec![];
                for _ in &unnamed.unnamed {
                    vars.push(quote!(
                        self.#i.push_to(map);
                    ));
                    i += 1;
                }
                Ok(quote!(
                    use #span::util::value::PushToValueMap;
                    #(#vars)*
                ))
            }
        },
        Data::Union(_) => Err(Error::new(Span::call_site(), "union not supportted")),
        Data::Enum(data) => {
            let mut vars = vec![];
            for var in &data.variants {
                let id = &var.ident;
                match &var.fields {
                    Fields::Named(named) => {
                        let ids = named.named.iter().collect::<Vec<_>>();
                        let fields = named_push_idents(&named, quote!(i))?;
                        vars.push(quote!(
                            Self::#id{#(#ids)*} => {
                                #fields
                            }
                        ))
                    }
                    Fields::Unit => vars.push(quote!(Self::#id => {})),
                    Fields::Unnamed(unnamed) => {
                        let mut i = 0;
                        let mut ids = vec![];
                        let mut fs = vec![];
                        for _ in &unnamed.unnamed {
                            let id = Ident::new(&format!("v{}", i), Span::call_site());
                            ids.push(id.clone());
                            fs.push(quote!(#id.push_to(map);));
                            i += 1;
                        }
                        vars.push(quote!(Self::#id(#(#ids)*) => {
                            use #span::util::value::PushToValueMap;
                            #(#fs)*
                        }))
                    }
                }
            }
            Ok(quote!(match self {
                #(#vars)*
            }))
        }
    }
}

fn named_push_idents(named: &FieldsNamed, head: TokenStream2) -> Result<TokenStream2> {
    let mut out = quote!();
    for field in &named.named {
        let i = field.ident.clone().unwrap();
        let mut s = i.to_string();
        escape(&mut s);
        out.extend(quote!(map.insert(#s.to_string(), #head.#i.into());))
    }
    Ok(out)
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
