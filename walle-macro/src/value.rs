use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result};

use crate::{error, escape, fields_from_map};

pub(crate) fn try_from_value_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = input.ident;

    let fields = if let Data::Struct(data) = input.data {
        fields_from_map(&data.fields)
    } else {
        return Err(error("only support struct"));
    };

    Ok(quote!(
        impl TryFrom<&mut #span::util::value::ValueMap> for #name {
            type Error = #span::WalleError;
            fn try_from(map: &mut #span::util::value::ValueMap) -> Result<Self, Self::Error> {
                use #span::util::value::ValueMapExt;
                Ok(Self #fields)
            }
        }

        impl TryFrom<#span::util::value::Value> for #name {
            type Error =  #span::WalleError;
            fn try_from(value: #span::util::value::Value) -> Result<Self, Self::Error> {
                match value {
                    #span::util::value::Value::Map(mut map) => Self::try_from(&mut map),
                    v => Err(#span::WalleError::DeclareNotMatch("Map Value", format!("{:?}", v))),
                }
            }
        }
    ))
}

pub(crate) fn push_to_value_map_internal(
    input: DeriveInput,
    span: TokenStream2,
) -> Result<TokenStream2> {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(named) => {
                let mut stream = TokenStream2::new();
                for field in named.named.iter() {
                    let name = field.ident.clone().unwrap();
                    let mut s = name.to_string();
                    escape(&mut s);
                    stream.extend(quote!(map.insert(#s.to_string(), self.#name.into());));
                }
                stream
            }
            Fields::Unnamed(unamed) => {
                let mut stream = quote!(use #span::util::value::PushToValueMap;);
                for i in 0..unamed.unnamed.len() {
                    let index = syn::Index::from(i);
                    stream.extend(quote!(self.#index.push_to(map);))
                }
                stream
            }
            Fields::Unit => TokenStream2::new(),
        },
        Data::Enum(data) => {
            let v = data
                .variants
                .into_iter()
                .map(|var| {
                    let vname = var.ident;
                    match var.fields {
                        Fields::Named(named) => {
                            let idents = named
                                .named
                                .into_iter()
                                .map(|f| f.ident.unwrap())
                                .collect::<Vec<_>>();
                            let ss = idents
                                .iter()
                                .map(|i| {
                                    let mut s = i.to_string();
                                    escape(&mut s);
                                    s
                                })
                                .collect::<Vec<_>>();
                            quote!(Self::#vname{#(#idents),*} => {
                                #(map.insert(#ss.to_string(), #idents.into());)*
                            })
                        }
                        Fields::Unnamed(unamed) => {
                            let idents = (0..unamed.unnamed.len())
                                .map(|i| Ident::new(&format!("f{}", i), Span::call_site()))
                                .collect::<Vec<_>>();
                            quote!(Self::#vname(#(#idents),*) => {
                                use #span::util::value::PushToValueMap;
                                #(#idents.push_to(map);)*
                            })
                        }
                        Fields::Unit => quote!(Self::#vname => {}),
                    }
                })
                .collect::<Vec<_>>();
            quote!(match self {
                #(#v)*
            })
        }
        Data::Union(_) => quote!(),
    };
    Ok(quote!(
        impl #span::util::value::PushToValueMap for #name {
            fn push_to(self, map: &mut #span::util::value::ValueMap)
            where
                Self: Sized,
            {
                #fields
            }
        }
        impl From<#name> for #span::util::value::ValueMap {
            fn from(s: #name) -> Self {
                use #span::util::value::PushToValueMap;
                let mut map = #span::util::value::ValueMap::default();
                s.push_to(&mut map);
                map
            }
        }
        impl From<#name> for #span::util::value::Value {
            fn from(s: #name) -> Self {
                #span::util::value::Value::Map(s.into())
            }
        }
    ))
}
