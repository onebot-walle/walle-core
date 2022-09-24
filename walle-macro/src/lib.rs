use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Error, Fields, Result, Type};

mod action_segment;
mod event;
mod value;

fn flatten(input: Result<TokenStream2>) -> TokenStream2 {
    match input {
        Ok(stream) => stream,
        Err(e) => e.into_compile_error(),
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

use event::try_from_event_internal;

ob!(TryFromEvent: event => try_from_event, try_from_event_internal, walle_core);
ob!(_TryFromEvent: event => _try_from_event, try_from_event_internal, crate);

use action_segment::to_action_internal;

ob!(ToAction: action => to_action, to_action_internal, walle_core);
ob!(_ToAction: action => _to_action, to_action_internal, crate);

use action_segment::to_msg_segment_internal;

ob!(ToMsgSegment: msg_segment => to_msg_segment, to_msg_segment_internal, walle_core);
ob!(_ToMsgSegment: msg_segment => _to_msg_segment, to_msg_segment_internal, crate);

use action_segment::try_from_action_internal;

ob!(TryFromAction: action => try_from_action, try_from_action_internal, walle_core);
ob!(_TryFromAction: action => _try_from_action, try_from_action_internal, crate);

use action_segment::try_from_msg_segment_internal;

ob!(TryFromMsgSegment: msg_segment => try_from_msg_segment, try_from_msg_segment_internal, walle_core);
ob!(_TryFromMsgSegment: msg_segment => _try_from_msg_segment, try_from_msg_segment_internal, crate);

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
                    quote!(#t::try_from(map)?)
                })
                .collect::<Vec<_>>();
            quote!((#(#v),*))
        }
        Fields::Unit => quote!(),
    }
}
