use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Ident, Lit, Meta, NestedMeta, Result};

use crate::fields_from_map;

use super::snake_case;

pub(crate) fn to_msg_segment_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = &input.ident;
    let extra = quote!(
        impl From<#name> for #span::segment::MsgSegment {
            fn from(v: #name) -> #span::segment::MsgSegment {
                use #span::segment::ToMsgSegment;
                v.to_segment()
            }
        }
    );
    to_internal(input, quote!(#span::segment::ToMsgSegment), "msg_segment").map(|mut s| {
        s.extend(extra);
        s
    })
}

pub(crate) fn to_action_internal(input: DeriveInput, span: TokenStream2) -> Result<TokenStream2> {
    let name = &input.ident;
    let extra = quote!(
        impl From<#name> for #span::action::Action {
            fn from(v: #name) -> #span::action::Action {
                use #span::action::ToAction;
                v.to_action()
            }
        }
    );
    to_internal(input, quote!(#span::action::ToAction), "action").map(|mut s| {
        s.extend(extra);
        s
    })
}

fn to_internal(input: DeriveInput, trait_name: TokenStream2, ty: &str) -> Result<TokenStream2> {
    let name = input.ident;
    match input.data {
        Data::Struct(_) | Data::Union(_) => {
            let s = attrs_parse(&name, &input.attrs, ty)?;
            Ok(quote!(
                impl #trait_name for #name {
                    fn ty(&self) -> &'static str {
                        #s
                    }
                }
            ))
        }
        Data::Enum(data) => {
            let v = data
                .variants
                .into_iter()
                .map(|v| {
                    let vname = v.ident;
                    // todo attr
                    let s = snake_case(vname.to_string());
                    match v.fields {
                        Fields::Named(_) => quote!(Self::#vname {..} => #s),
                        Fields::Unnamed(_) => quote!(Self::#vname (..) => #s),
                        Fields::Unit => quote!(Self::#vname => #s),
                    }
                })
                .collect::<Vec<_>>();
            Ok(quote!(impl #trait_name for #name {
                fn ty(&self) -> &'static str {
                    match self {
                        #(#v,)*
                    }
                }
            }))
        }
    }
}

pub(crate) fn try_from_msg_segment_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = &input.ident;
    let extra = quote!(
        impl TryFrom<#span::segment::MsgSegment> for #name {
            type Error = #span::WalleError;
            fn try_from(segment: #span::segment::MsgSegment) -> Result<Self, Self::Error> {
                use #span::segment::TryFromMsgSegment;
                Self::try_from_msg_segment(segment)
            }
        }
    );
    try_from_internal(
        input,
        quote!(#span::segment::TryFromMsgSegment),
        quote!(try_from_msg_segment_mut(segment: &mut #span::segment::MsgSegment)),
        quote!(segment.ty),
        quote!(segment.data),
        span,
        "msg_sgement",
    )
    .map(|mut s| {
        s.extend(extra);
        s
    })
}

pub(crate) fn try_from_action_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = &input.ident;
    let extra = quote!(
        impl TryFrom<#span::action::Action> for #name {
            type Error = #span::WalleError;
            fn try_from(action: #span::action::Action) -> Result<Self, Self::Error> {
                use #span::action::TryFromAction;
                Self::try_from_action(action)
            }
        }
    );
    try_from_internal(
        input,
        quote!(#span::action::TryFromAction),
        quote!(try_from_action_mut(action: &mut #span::action::Action)),
        quote!(action.action),
        quote!(action.params),
        span,
        "msg_sgement",
    )
    .map(|mut s| {
        s.extend(extra);
        s
    })
}

fn try_from_internal(
    input: DeriveInput,
    trait_name: TokenStream2,
    f: TokenStream2,
    field: TokenStream2,
    map: TokenStream2,
    span: TokenStream2,
    ty: &str,
) -> Result<TokenStream2> {
    let name = input.ident;
    match input.data {
        Data::Union(_) => {
            let s = attrs_parse(&name, &input.attrs, ty)?;
            Ok(quote!(impl #trait_name for #name {
                fn #f -> #span::WalleResult<Self> {
                    if #field.as_str() == #s {
                        Ok(Self)
                    } else {
                        Err(#span::WalleError::DeclareNotMatch(#s, #field.to_string()))
                    }
                }
            }))
        }
        Data::Struct(data) => {
            let s = attrs_parse(&name, &input.attrs, ty)?;
            let fs = fields_from_map(&data.fields);
            Ok(quote!(impl #trait_name for #name {
                fn #f -> #span::WalleResult<Self> {
                    use #span::util::value::ValueMapExt;
                    let map = &mut #map;
                    if #field.as_str() == #s {
                        Ok(Self #fs)
                    } else {
                        Err(#span::WalleError::DeclareNotMatch(#s, #field.to_string()))
                    }
                }
            }))
        }
        Data::Enum(data) => {
            let mut ss = Vec::new();
            let v = data
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
            let ss = ss.join("|");
            Ok(quote!(
                impl #trait_name for #name {
                    fn #f -> #span::WalleResult<Self> {
                        use #span::util::value::ValueMapExt;
                        let map = &mut #map;
                        match #field.as_str() {
                            #(#v,)*
                            _ => Err(#span::WalleError::DeclareNotMatch(#ss, #field.to_string()))
                        }
                    }
                }
            ))
        }
    }
}

fn attrs_parse(name: &Ident, attrs: &Vec<Attribute>, ty: &str) -> Result<String> {
    for attr in attrs {
        if attr.path.is_ident(ty) {
            match attr.parse_meta()? {
                Meta::NameValue(v) => {
                    if let Lit::Str(s) = v.lit {
                        if v.path.is_ident("rename") {
                            return Ok(s.value());
                        }
                    }
                }
                Meta::List(l) => {
                    for nest in l.nested {
                        if let NestedMeta::Lit(Lit::Str(s)) = nest {
                            return Ok(s.value());
                        } else if let NestedMeta::Meta(Meta::NameValue(v)) = nest {
                            if let Lit::Str(s) = v.lit {
                                if v.path.is_ident("rename") {
                                    return Ok(s.value());
                                }
                            }
                        }
                    }
                }
                Meta::Path(_) => {}
            }
        }
    }
    Ok(snake_case(name.to_string()))
}
