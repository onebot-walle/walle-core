use crate::{action::*, ActionResp, EmptyContent};

macro_rules! action_api {
    ($fn_name: ident,$action_ty:ident,$content:ident) => {
        pub async fn $fn_name(&self) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content::default())).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $field_name: ident: $field_ty: ty) => {
        pub async fn $fn_name(&self, $field_name: $field_ty) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content{
                $field_name,
            })).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $($field_name: ident: $field_ty: ty),*) => {
        pub async fn $fn_name(&self, $($field_name: $field_ty,)*) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content{
                $($field_name,)*
            })).await
        }
    };
}

impl<E, R> super::CustomOneBot<E, Action, R>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    pub async fn get_latest_events(&self, limit: i64, timeout: i64) -> Option<ActionResp<R>> {
        self.call_action_resp(Action::GetLatestEvents(GetLatestEventsContent {
            limit,
            timeout,
        }))
        .await
    }
    action_api!(get_support_actions, GetSupportedActions, EmptyContent);
    action_api!(get_status, GetStatus, EmptyContent);
    action_api!(get_vetsion, GetVersion, EmptyContent);

    action_api!(
        send_message,
        SendMessage,
        SendMessageContent,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: crate::Message
    );
    action_api!(
        deletemessage,
        DeleteMessage,
        DeleteMessageContent,
        message_id: String
    );

    action_api!(get_self_info, GetSelfInfo, EmptyContent);
    action_api!(get_user_info, GetUserInfo, UserIdContent, user_id: String);
}
