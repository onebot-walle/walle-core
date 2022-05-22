use crate::message::MessageSegment as PyMsgSeg;
use pyo3::prelude::*;
// use walle_core::StandardEvent;

#[pyclass(subclass)]
pub struct Event {
    #[pyo3(get, set)]
    pub r#impl: String,
    #[pyo3(get, set)]
    pub platform: String,
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub self_id: String,
    #[pyo3(get, set)]
    pub time: f64,
}

#[pymethods]
impl Event {
    #[new]
    pub fn new(r#impl: String, platform: String, id: String, self_id: String, time: f64) -> Self {
        Self {
            r#impl,
            platform,
            id,
            self_id,
            time,
        }
    }
}

#[pyclass(extends = Event, subclass)]
pub struct MessageEvent {
    #[pyo3(get, set)]
    pub message_id: String,
    #[pyo3(get, set)]
    pub alt_message: String,
    #[pyo3(get, set)]
    pub message: Vec<PyMsgSeg>,
    #[pyo3(get, set)]
    pub user_id: String,
    #[pyo3(get, set)]
    pub sub_type: String,
}

#[pymethods]
impl MessageEvent {
    #[new]
    pub fn new(
        r#impl: String,
        platform: String,
        id: String,
        self_id: String,
        time: f64,
        message_id: String,
        alt_message: String,
        message: Vec<PyMsgSeg>,
        user_id: String,
        sub_type: String,
    ) -> (Self, Event) {
        (
            Self {
                message_id,
                alt_message,
                message,
                user_id,
                sub_type,
            },
            Event::new(r#impl, platform, id, self_id, time),
        )
    }
}

#[pyclass(extends = MessageEvent, subclass)]
pub struct PrivateMessageEvent;

#[pymethods]
impl PrivateMessageEvent {
    #[new]
    pub fn new(
        r#impl: String,
        platform: String,
        id: String,
        self_id: String,
        time: f64,
        message_id: String,
        alt_message: String,
        message: Vec<PyMsgSeg>,
        user_id: String,
        sub_type: String,
    ) -> PyClassInitializer<Self> {
        PyClassInitializer::from(MessageEvent::new(
            r#impl,
            platform,
            id,
            self_id,
            time,
            message_id,
            alt_message,
            message,
            user_id,
            sub_type,
        ))
        .add_subclass(Self)
    }
}

// pub struct PyStandardEvent(StandardEvent);

// impl IntoPy<PyObject> for PyStandardEvent {
//     fn into_py(self, py: Python) -> PyObject {
//         match self.0.content {
//             EventContent::Message(c) => match c.ty {
//                 MessageEventType::Private => PrivateMessageEvent.into_py(py),
//             },
//         }
//     }
// }
