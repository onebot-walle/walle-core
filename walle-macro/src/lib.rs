use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Lit, Meta, NestedMeta, Result};

#[proc_macro_derive(EventContent, attributes(event, internal))]
pub fn onebot_event(token: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(token as DeriveInput);
    let mut info = TypeInfo {
        name: input.ident.clone(),
        ty: None,
        detail_type: None,
        sub_type: None,
        implt: None,
        platform: None,
        span: quote!(walle_core),
    };
    let token: TokenStream2 = match declare(&input, &mut info) {
        Ok(token) => token,
        Err(e) => e.into_compile_error(),
    };
    let token2: TokenStream2 = match transfer(&input, &info) {
        Ok(token) => token,
        Err(e) => e.into_compile_error(),
    };
    quote!(#token #token2).into()
}

struct TypeInfo {
    pub name: Ident,
    pub ty: Option<String>,
    pub detail_type: Option<String>,
    pub sub_type: Option<String>,
    pub implt: Option<String>,
    pub platform: Option<String>,
    pub span: TokenStream2,
}

impl TypeInfo {
    fn build_impl(&self) -> TokenStream2 {
        let name = &self.name;
        let span = &self.span;
        let mut token = quote!();
        if let Some(s) = &self.ty {
            token = quote!(
                #token
                impl #span ::event::next::TypeDeclare for #name {
                    fn ty() -> &'static str {
                        #s
                    }
                }
            );
        }
        if let Some(s) = &self.detail_type {
            token = quote!(
                #token
                impl #span ::event::next::DetailTypeDeclare for #name {
                    fn detail_type() -> &'static str {
                        #s
                    }
                }
            );
        }
        if let Some(s) = &self.sub_type {
            token = quote!(
                #token
                impl #span ::event::next::SubTypeDeclare for #name {
                    fn sub_type() -> &'static str {
                        #s
                    }
                }
            );
        }
        if let Some(s) = &self.implt {
            token = quote!(
                #token
                impl #span ::event::next::ImplDeclare for #name {
                    fn implt() -> &'static str {
                        #s
                    }
                }
            );
        }
        if let Some(s) = &self.platform {
            token = quote!(
                #token
                impl #span ::event::next::PlatformDeclare for #name {
                    fn platform() -> &'static str {
                        #s
                    }
                }
            );
        }
        token
    }

    fn build_check(&self) -> TokenStream2 {
        let mut token = quote!();
        let span = &self.span;
        if let Some(s) = &self.ty {
            token = quote!(
                #token
                if e.ty != #s {
                    return Err(#span ::error::WalleError::EventDeclareNotMatch(
                        #s,
                        e.ty.clone(),
                ))}
            )
        }
        if let Some(s) = &self.detail_type {
            token = quote!(
                #token
                if e.detail_type != #s {
                    return Err(#span ::error::WalleError::EventDeclareNotMatch(
                        #s,
                        e.detail_type.clone(),
                ))}
            )
        }
        if let Some(s) = &self.sub_type {
            token = quote!(
                #token
                if e.sub_type != #s {
                    return Err(#span ::error::WalleError::EventDeclareNotMatch(
                        #s,
                        e.sub_type.clone(),
                ))}
            )
        }
        if let Some(s) = &self.implt {
            token = quote!(
                #token
                if e.implt != #s {
                    return Err(#span ::error::WalleError::EventDeclareNotMatch(
                        #s,
                        e.implt.clone(),
                ))}
            )
        }
        if let Some(s) = &self.platform {
            token = quote!(
                #token
                if e.platform != #s {
                    return Err(#span ::error::WalleError::EventDeclareNotMatch(
                        #s,
                        e.platform.clone(),
                ))}
            )
        }
        token
    }
}

fn declare(input: &DeriveInput, info: &mut TypeInfo) -> Result<TokenStream2> {
    for attr in &input.attrs {
        if attr.path.is_ident("event") {
            if let Meta::List(l) = attr.parse_meta()? {
                for nmeta in l.nested {
                    if let NestedMeta::Meta(Meta::NameValue(v)) = nmeta {
                        if let Lit::Str(str) = v.lit {
                            match v.path.get_ident().unwrap().to_string().as_str() {
                                "type" => info.ty = Some(str.value()),
                                "detail_type" => info.detail_type = Some(str.value()),
                                "sub_type" => info.sub_type = Some(str.value()),
                                "platform" => info.platform = Some(str.value()),
                                "impl" => info.implt = Some(str.value()),
                                _ => return Err(Error::new(Span::call_site(), "unkown type")),
                            }
                        }
                    }
                }
            }
        }
        if attr.path.is_ident("internal") {
            info.span = quote!(crate)
        }
    }
    Ok(info.build_impl())
}

fn transfer(input: &DeriveInput, info: &TypeInfo) -> Result<TokenStream2> {
    let name = &info.name;
    let span = &info.span;
    if let Data::Struct(data) = &input.data {
        if let Fields::Named(v) = &data.fields {
            let mut idents = vec![];
            let mut ident_strs = vec![];
            for field in &v.named {
                let i = field.ident.clone().unwrap();
                ident_strs.push(i.to_string());
                idents.push(i);
            }
            let check = info.build_check();
            Ok(quote!(
                impl TryFrom<&mut #span ::event::next::Event> for #name {
                    type Error = #span ::error::WalleError;
                    fn try_from(e: &mut #span ::event::next::Event) -> Result<Self, Self::Error> {
                        use #span ::util::value::ExtendedMapExt;
                        #check
                        Ok(Self {
                            #(#idents: e.extra.remove_downcast(#ident_strs)?,)*
                        })
                    }
                }

                impl #span ::util::value::PushToExtendedMap for #name {
                    fn push(self, map: &mut #span ::util::value::ExtendedMap) {
                        #(map.insert(#ident_strs.to_string(), self.#idents.into());)*
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
        } else {
            Err(Error::new(Span::call_site(), "expect named struct"))
        }
    } else {
        Err(Error::new(Span::call_site(), "expect struct"))
    }
}
