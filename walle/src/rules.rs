use walle_core::{EventContent, MessageEventType, StandardEvent};

pub fn check_user_id(event: &StandardEvent, user_id: &str) -> bool {
    if let EventContent::Message(c) = &event.content {
        if c.user_id == user_id {
            return true;
        }
    }
    false
}

pub fn check_group_id(event: &StandardEvent, group_id: &str) -> bool {
    if let EventContent::Message(c) = &event.content {
        if let MessageEventType::Group { group_id: id } = &c.ty {
            if id == group_id {
                return true;
            }
        }
    }
    false
}

pub fn check_both_id(event: &StandardEvent, user_id: &str, group_id: &str) -> bool {
    if let EventContent::Message(c) = &event.content {
        if c.user_id == user_id {
            if let MessageEventType::Group { group_id: id } = &c.ty {
                if id == group_id {
                    return true;
                }
            }
        }
    }
    false
}
