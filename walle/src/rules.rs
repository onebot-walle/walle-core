use walle_core::{EventContent, MessageEventType, StandardEvent};

pub fn check_user_id(user_id: &str) -> impl Fn(&StandardEvent) -> bool {
    let user_id = user_id.to_owned();
    move |event: &StandardEvent| {
        if let EventContent::Message(ref c) = event.content {
            if &c.user_id == &user_id {
                return true;
            }
        }
        false
    }
}

pub fn check_group_id(group_id: &str) -> impl Fn(&StandardEvent) -> bool {
    let group_id = group_id.to_owned();
    move |event: &StandardEvent| {
        if let EventContent::Message(c) = &event.content {
            if let MessageEventType::Group { group_id: id } = &c.ty {
                if id == &group_id {
                    return true;
                }
            }
        }
        false
    }
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

#[test]
fn rule_vec_test() {
    let _: Vec<Box<dyn Fn(&StandardEvent) -> bool>> = vec![
        Box::new(check_user_id("user_id")),
        Box::new(check_group_id("group_id")),
    ];
}
