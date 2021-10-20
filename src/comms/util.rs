pub enum ContentTpye {
    Json,
    MsgPack,
}

impl ContentTpye {
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}
