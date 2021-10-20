use serde::Deserialize;

pub fn parse_event(s: &str) -> Option<crate::Events> {
    match serde_json::from_str::<crate::Events>(s) {
        Ok(e) => Some(e),
        Err(_) => None,
    }
}

#[cfg(not(feature = "echo"))]
pub fn parse_action(s: &str) -> Option<crate::Action> {
    match serde_json::from_str::<crate::Action>(s) {
        Ok(a) => Some(a),
        Err(_) => None,
    }
}

#[cfg(feature = "echo")]
pub fn parse_action(s: &str) -> Option<crate::EchoAction> {
    match serde_json::from_str::<crate::EchoAction>(s) {
        Ok(a) => Some(a),
        Err(_) => None,
    }
}

#[cfg(not(feature = "echo"))]
pub fn parse_action_resp<'de, T>(s: &'de str) -> Option<crate::ActionResp<T>>
where
    T: Deserialize<'de>,
{
    match serde_json::from_str::<crate::ActionResp<T>>(s) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

#[cfg(feature = "echo")]
pub fn parse_action_resp<'de, T>(s: &'de str) -> Option<crate::EchoActionResp<T>>
where
    T: Deserialize<'de>,
{
    match serde_json::from_str::<crate::EchoActionResp<T>>(s) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}
