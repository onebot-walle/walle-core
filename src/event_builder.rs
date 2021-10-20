// use crate::Events;

#[derive(Clone)]
pub struct Builder {
    r#impl: String,
    platform: String,
    self_id: String,
}

impl Builder {
    /// new EventBuilder
    #[allow(dead_code)]
    pub fn new(r#impl: &str, platform: &str, self_id: &str) -> Self {
        Builder {
            r#impl: r#impl.to_owned(),
            platform: platform.to_owned(),
            self_id: self_id.to_owned(),
        }
    }

    //pub fn build_events(content: crate::event::EventContent) -> Events {}
}
