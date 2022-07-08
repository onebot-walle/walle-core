use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Error, Fields, Lit, Meta, NestedMeta, Result, Type};

#[proc_macro_derive(OneBot, attributes(event, action))]
pub fn onebot(token: TokenStream) -> TokenStream {
    onebot_internal(token, quote!(walle_core))
}

#[proc_macro_derive(_OneBot, attributes(event, action))]
pub fn _onebot(token: TokenStream) -> TokenStream {
    onebot_internal(token, quote!(crate))
}

fn onebot_internal(token: TokenStream, span: TokenStream2) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    let mut stream = quote!();
    for attr in &input.attrs {
        if attr.path.is_ident("event") {
            let mut info = EventTypeInfo {
                name: input.ident.clone(),
                ty: None,
                detail_type: None,
                sub_type: None,
                implt: None,
                platform: None,
            };
            stream.extend(flatten(event_declare(attr, &mut info, &span)));
            stream.extend(flatten(event_impl(&input, &info, &span)));
        } else if attr.path.is_ident("action") {
            stream.extend(flatten(action_impl(attr, &input, &span)))
        }
    }
    stream.into()
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
        impl #span ::util::value::PushToExtendedMap for #name {
            fn push(self, map: &mut #span ::util::value::ExtendedMap) {
                #(#idents)*
            }
        }

        impl Into<#span ::util::value::ExtendedValue> for #name {
            fn into(self) -> #span ::util::value::ExtendedValue {
                use #span ::util::value::PushToExtendedMap;
                let mut map = #span ::util::value::ExtendedMap::default();
                self.push(&mut map);
                #span ::util::value::ExtendedValue::Map(map)
            }
        }
    ))
}

#[proc_macro_derive(TryFromValue)]
pub fn from_value(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(from_value_internal(input, quote!(walle_core))).into()
}

#[proc_macro_derive(_TryFromValue)]
pub fn _from_value(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    flatten(from_value_internal(input, quote!(crate))).into()
}

fn from_value_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    let idents = try_from_idents(&input, quote!(map))?;
    Ok(quote!(
        impl TryFrom<#span::util::value::ExtendedValue> for #name {
            type Error = #span::error::WalleError;
            fn try_from(v: #span::util::value::ExtendedValue) -> Result<Self, Self::Error> {
                if let #span::util::value::ExtendedValue::Map(mut map) = v {
                    Ok(Self {
                        #(#idents)*
                    })
                } else {
                    Err(#span::error::WalleError::ValueTypeNotMatch(
                        "map".to_string(),
                        format!("{:?}", v),
                    ))
                }
            }
        }
    ))
}

fn flatten(input: Result<TokenStream2>) -> TokenStream2 {
    match input {
        Ok(stream) => stream,
        Err(e) => e.into_compile_error(),
    }
}

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
                impl #span ::event::next::TypeDeclare for #name {
                    fn ty() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.detail_type {
            token.extend(quote!(
                impl #span ::event::next::DetailTypeDeclare for #name {
                    fn detail_type() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.sub_type {
            token.extend(quote!(
                impl #span ::event::next::SubTypeDeclare for #name {
                    fn sub_type() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.implt {
            token.extend(quote!(
                impl #span ::event::next::ImplDeclare for #name {
                    fn implt() -> &'static str {
                        #s
                    }
                }
            ));
        }
        if let Some(s) = &self.platform {
            token.extend(quote!(
                impl #span ::event::next::PlatformDeclare for #name {
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

fn event_impl(
    input: &DeriveInput,
    info: &EventTypeInfo,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    let name = &info.name;
    let idents = try_from_idents(input, quote!(e.extra))?;
    let check = info.build_check(span);
    Ok(quote!(
        impl TryFrom<&mut #span ::event::next::Event> for #name {
            type Error = #span ::error::WalleError;
            fn try_from(e: &mut #span ::event::next::Event) -> Result<Self, Self::Error> {
                use #span ::util::value::ExtendedMapExt;
                #check
                Ok(Self {
                    #(#idents)*
                })
            }
        }
    ))
}

fn action_impl(attr: &Attribute, input: &DeriveInput, span: &TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    let s = match attr.parse_meta()? {
        Meta::Path(_) => snake_case(name.to_string()),
        Meta::NameValue(v) => match v.lit {
            Lit::Str(s) => s.value(),
            _ => return Err(Error::new(Span::call_site(), "expect string for action")),
        },
        _ => return Err(Error::new(Span::call_site(), "expect NameValue for action")),
    };
    let fields = try_from_idents(input, quote!(a.params))?;
    // let (idents, ident_strs) = get_fields(input)?;
    Ok(quote!(
        impl #span ::action_next::ActionDeclare for #name {
            fn action() -> &'static str {
                #s
            }
        }

        impl TryFrom<&mut #span ::action_next::Action> for #name {
            type Error = #span ::error::WalleError;
            fn try_from(a: &mut #span ::action_next::Action) -> Result<Self, Self::Error> {
                use #span ::action_next::ActionDeclare;
                use #span ::util::value::ExtendedMapExt;
                if a.action.as_str() != Self::action() {
                    Err(#span::error::WalleError::DeclareNotMatch(
                        Self::action(),
                        a.action.clone(),
                    ))
                } else {
                    Ok(Self {
                        #(#fields)*
                    })
                }
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

fn try_from_idents(input: &DeriveInput, head: TokenStream2) -> Result<Vec<TokenStream2>> {
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(v) = &data.fields {
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
                            #ident: #head.try_remove_downcast(#s)?,
                        ));
                        continue;
                    }
                }
                out.push(quote!(
                    #ident: #head.remove_downcast(#s)?,
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
