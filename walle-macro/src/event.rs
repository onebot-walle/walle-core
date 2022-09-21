use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Lit, Meta, NestedMeta,
    Result,
};

use crate::error;

use super::{snake_case, try_from_idents};

#[derive(Debug, Clone, Copy)]
enum ContentType {
    Type,
    DetailType,
    SubType,
    Platform,
    Impl,
}

impl TryFrom<&str> for ContentType {
    type Error = Error;
    fn try_from(s: &str) -> Result<Self> {
        match s {
            "type" => Ok(ContentType::Type),
            "detail_type" => Ok(ContentType::DetailType),
            "sub_type" => Ok(ContentType::SubType),
            "platform" => Ok(ContentType::Platform),
            "impl" => Ok(ContentType::Impl),
            _ => Err(Error::new(Span::call_site(), "unkown event content type")),
        }
    }
}

impl ContentType {
    pub(crate) fn traitt(&self, span: &TokenStream2) -> TokenStream2 {
        match self {
            ContentType::Type => quote!(#span::event::TypeDeclare),
            ContentType::DetailType => quote!(#span::event::DetailTypeDeclare),
            ContentType::SubType => quote!(#span::event::SubTypeDeclare),
            ContentType::Platform => quote!(#span::event::PlatformDeclare),
            ContentType::Impl => quote!(#span::event::ImplDeclare),
        }
    }
    pub(crate) fn traitf(&self) -> TokenStream2 {
        match self {
            ContentType::Type => quote!(ty),
            ContentType::DetailType => quote!(detail_type),
            ContentType::SubType => quote!(sub_type),
            ContentType::Platform => quote!(platform),
            ContentType::Impl => quote!(implt),
        }
    }
    pub(crate) fn traiti(&self) -> TokenStream2 {
        if let Self::Platform = self {
            quote!(platform().unwrap_or_default())
        } else {
            self.traitf()
        }
    }
    pub(crate) fn struct_declare(
        &self,
        name: &Ident,
        span: &TokenStream2,
        s: &str,
        fields: &Fields,
    ) -> Result<TokenStream2> {
        let idents = try_from_idents(&fields, quote!(e.extra), true)?;
        let t = self.traitt(span);
        let f = self.traitf();
        if let Self::Impl = self {
            return Ok(quote!(
                impl #t for #name {
                    fn #f(&self) -> &'static str {
                        #s
                    }
                    fn check(implt: &str) -> bool {
                        implt == #s
                    }
                    fn from_event(e: &mut #span::event::Event, implt: &str) -> Result<Self, #span::error::WalleError>
                        where Self: Sized
                    {
                        use #span::util::value::ValueMapExt;
                        if implt == #s {
                            Ok(Self #idents)
                        } else {
                            return Err(#span::error::WalleError::DeclareNotMatch(
                                #s,
                                implt.to_owned()
                            ));
                        }
                    }
                }
            ));
        }
        let i = self.traiti();
        Ok(quote!(
            impl #t for #name {
                fn #f(&self) -> &'static str {
                    #s
                }
                fn check(event: &#span::event::Event) -> bool {
                    event.#i.as_str() == #s
                }
            }
            impl TryFrom<&mut #span::event::Event> for #name {
                type Error = #span::error::WalleError;
                fn try_from(e: &mut #span::event::Event) -> Result<Self, Self::Error> {
                    use #span::util::value::ValueMapExt;
                    use #t;
                    if !Self::check(e) {
                        return Err(#span::error::WalleError::DeclareNotMatch(
                            #s,
                            e.#i.clone()
                        ));
                    }
                    Ok(Self #idents)
                }
            }
            impl TryFrom<#span::event::Event> for #name {
                type Error = #span::error::WalleError;
                fn try_from(mut e: #span::event::Event) -> Result<Self, Self::Error> {
                    Self::try_from(&mut e)
                }
            }
        ))
    }
}

pub(crate) fn event_internal(
    attr: &Attribute,
    input: &DeriveInput,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    match &input.data {
        Data::Struct(data) => struct_declare(&input.ident, data, attr, span),
        Data::Enum(data) => enum_declare(&input.ident, data, attr, span),
        _ => return Err(Error::new(Span::call_site(), "union not supported")),
    }
}

fn struct_declare(
    name: &Ident,
    data: &DataStruct,
    attr: &Attribute,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    if let Meta::List(l) = attr.parse_meta()? {
        if l.nested.len() != 1 {
            return Err(Error::new(Span::call_site(), "only support one nested"));
        }
        let nmeta = l.nested.first().unwrap();
        let (s, path) = match &nmeta {
            NestedMeta::Meta(Meta::NameValue(v)) => {
                if let Lit::Str(s) = &v.lit {
                    (s.value(), &v.path)
                } else {
                    return Err(Error::new(Span::call_site(), "unsupport attributes"));
                }
            }
            NestedMeta::Meta(Meta::Path(p)) => (snake_case(name.to_string()), p),
            _ => return Err(Error::new(Span::call_site(), "unsupport attributes")),
        };
        let content = ContentType::try_from(path.get_ident().unwrap().to_string().as_str())?;
        content.struct_declare(name, span, &s, &data.fields)
    } else {
        Err(Error::new(Span::call_site(), "not metalist attributes"))
    }
}

fn enum_declare(
    name: &Ident,
    data: &DataEnum,
    attr: &Attribute,
    span: &TokenStream2,
) -> Result<TokenStream2> {
    if let Meta::List(l) = attr.parse_meta()? {
        if l.nested.len() != 1 {
            return Err(Error::new(Span::call_site(), "only support one nested"));
        }
        let nmeta = l.nested.first().unwrap();
        let content = match nmeta {
            NestedMeta::Meta(Meta::Path(p)) => {
                ContentType::try_from(p.get_ident().unwrap().to_string().as_str())?
            }
            _ => return Err(Error::new(Span::call_site(), "unsupport attributes")),
        };
        let mut declare_vars = vec![];
        let mut try_from_vars = vec![];
        let mut strs = vec![];
        for var in &data.variants {
            // todo attr
            let ident = &var.ident;
            let s = snake_case(ident.to_string());
            declare_vars.push(match &var.fields {
                Fields::Named(_) => quote!(Self::#ident{..} => #s),
                Fields::Unnamed(_) => quote!(Self::#ident(..) => #s),
                Fields::Unit => quote!(Self::#ident => #s),
            });
            let idents = try_from_idents(&var.fields, quote!(e), false)?;
            try_from_vars.push(quote!(#s => Ok(Self::#ident #idents)));
            strs.push(s);
        }
        let t = content.traitt(span);
        let f = content.traitf();
        let i = content.traiti();
        if let ContentType::Impl = content {
            Ok(quote!(
                impl #t for #name {
                    fn #f(&self) -> &'static str {
                        match self {
                            #(#declare_vars,)*
                        }
                    }
                    fn check(implt: &str) -> bool {
                        match implt {
                            #(#strs => true,)*
                            _ => false,
                        }
                    }
                    fn from_event(e: &mut Event, implt: &str) -> Result<Self, #span::error::WalleError>
                    where Self: Sized
                    {
                        use #span::util::value::ValueMapExt;
                        match implt {
                            #(#try_from_vars,)* // todo
                            _ => Err(#span::error::WalleError::DeclareNotMatch(
                                "event types",
                                implt.to_owned(),
                            ))
                        }
                    }
                }
            ))
        } else {
            Ok(quote!(
                impl #t for #name {
                    fn #f(&self) -> &'static str {
                        match self {
                            #(#declare_vars,)*
                        }
                    }
                    fn check(event: &#span::event::Event) -> bool {
                        match event.#i.as_str() {
                            #(#strs => true,)*
                            _ => false,
                        }
                    }
                }
                impl TryFrom<&mut #span::event::Event> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(e: &mut #span::event::Event) -> Result<Self, Self::Error> {
                        use #span::util::value::ValueMapExt;
                        match e.#f.as_str() {
                            #(#try_from_vars,)* // todo
                            _ => Err(#span::error::WalleError::DeclareNotMatch(
                                "event types",
                                e.#f.clone(),
                            ))
                        }
                    }
                }
                impl TryFrom<#span::event::Event> for #name {
                    type Error = #span::error::WalleError;
                    fn try_from(mut e: #span::event::Event) -> Result<Self, Self::Error> {
                        Self::try_from(&mut e)
                    }
                }
            ))
        }
    } else {
        Err(Error::new(Span::call_site(), "not metapath attributes"))
    }
}

pub(crate) fn to_event_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    todo!()
}

fn parse_ty(ty: &str, span: &TokenStream2) -> Result<TokenStream2> {
    match ty {
        "type" => Ok(quote!(#span::event::TypeLevel)),
        "detail_type" => Ok(quote!(#span::event::DetailTypeLevel)),
        "sub_type" => Ok(quote!(#span::event::SubTypeLevel)),
        "platform" => Ok(quote!(#span::event::PlatformLevel)),
        "impl" => Ok(quote!(#span::event::ImplLevel)),
        ty => Err(error(format!("unsupportted type {}", ty))),
    }
}
