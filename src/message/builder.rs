use super::MessageSegment;
use crate::util::ExtendedMap;

use super::Message;

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for String {
    fn into_message(self) -> Message {
        vec![MessageSegment::text(self)]
    }
}

impl IntoMessage for &str {
    fn into_message(self) -> Message {
        vec![MessageSegment::text(self.to_string())]
    }
}

impl IntoMessage for MessageSegment {
    fn into_message(self) -> Message {
        vec![self]
    }
}

impl IntoMessage for Message {
    fn into_message(self) -> Message {
        self
    }
}

macro_rules! impl_self_build {
    ($fname0: ident, $fname1: ident,$sub: tt) => {
        fn $fname0(mut self, extra: ExtendedMap) -> Self {
            self.push(MessageSegment::$sub { extra });
            self
        }
        fn $fname1(mut self) -> Self {
            self.push(MessageSegment::$sub { extra: ExtendedMap::new() });
            self
        }
    };
    ($fname0: ident, $fname1: ident,$sub: tt, $($key: ident: $key_ty: ty),*) => {
        fn $fname0(mut self, $($key: $key_ty),*, extra: ExtendedMap) -> Self {
            self.push(MessageSegment::$sub { $($key ,)* extra, });
            self
        }
        fn $fname1(mut self, $($key: $key_ty),*) -> Self {
            self.push(MessageSegment::$sub { $($key ,)* extra: ExtendedMap::new(), });
            self
        }
    };
}

/// Message 构建 trait
pub trait MessageBuild {
    fn text_with_extend(self, text: String, extra: ExtendedMap) -> Self;
    fn mention_with_extend(self, user_id: String, extra: ExtendedMap) -> Self;
    fn mention_all_with_extend(self, extra: ExtendedMap) -> Self;
    fn image_with_extend(self, file_id: String, extra: ExtendedMap) -> Self;
    fn voice_with_extend(self, file_id: String, extra: ExtendedMap) -> Self;
    fn audio_with_extend(self, file_id: String, extra: ExtendedMap) -> Self;
    fn video_with_extend(self, file_id: String, extra: ExtendedMap) -> Self;
    fn file_with_extend(self, file_id: String, extra: ExtendedMap) -> Self;
    fn location_with_extend(
        self,
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
        extra: ExtendedMap,
    ) -> Self;
    fn reply_with_extend(self, message_id: String, user_id: String, extra: ExtendedMap) -> Self;

    fn text(self, text: String) -> Self;
    fn mention(self, user_id: String) -> Self;
    fn mention_all(self) -> Self;
    fn image(self, file_id: String) -> Self;
    fn voice(self, file_id: String) -> Self;
    fn audio(self, file_id: String) -> Self;
    fn video(self, file_id: String) -> Self;
    fn file(self, file_id: String) -> Self;
    fn location(self, latitude: f64, longitude: f64, title: String, content: String) -> Self;
    fn reply(self, message_id: String, user_id: String) -> Self;
    fn custom(self, ty: String, data: ExtendedMap) -> Self;
}

impl MessageBuild for Message {
    impl_self_build!(text_with_extend, text, Text, text: String);
    impl_self_build!(mention_with_extend, mention, Mention, user_id: String);
    impl_self_build!(mention_all_with_extend, mention_all, MentionAll);
    impl_self_build!(image_with_extend, image, Image, file_id: String);
    impl_self_build!(voice_with_extend, voice, Voice, file_id: String);
    impl_self_build!(audio_with_extend, audio, Audio, file_id: String);
    impl_self_build!(video_with_extend, video, Video, file_id: String);
    impl_self_build!(file_with_extend, file, File, file_id: String);
    impl_self_build!(
        location_with_extend,
        location,
        Location,
        latitude: f64,
        longitude: f64,
        title: String,
        content: String
    );
    impl_self_build!(
        reply_with_extend,
        reply,
        Reply,
        message_id: String,
        user_id: String
    );
    fn custom(mut self, ty: String, data: ExtendedMap) -> Self {
        self.push(MessageSegment::Custom { ty, data });
        self
    }
}
