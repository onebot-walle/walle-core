# Event

以下使用 Event 代指符合 OneBot 标准，未反序列化的事件。

## BaseEvent

OneBot 中的事件类型固定为四种，禁止扩展 type

- Meta
- Message
- Notice
- Request 

由于不同 type 之间拥有不少共同字段，而 Rust Trait 只能约定方法而无法约定字段，因此所有的 Event 抽象拥有一个基本 struct `BaseEvent` 持有所有共同字段。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseEvent<T> {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: i64,
    #[serde(flatten)]
    pub content: T,
}
```

其余字段，均有 content 持有

## EventContent

EventContent 为一个标准的（无扩展的） content 枚举。

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    Meta(MetaContent),
    Message(MessageContent<MessageEventDetail>),
    Notice(NoticeContent),
    Request(RequestContent),
}
```

EventContent 使用 type 字段区分不同的类型，因此符合 Onebot 的标准事件可以反序列化为 `BaseEvent<EventContent>` 类型。

> 该类型可以直接使用类型别名（alias）: Event
> ```rust
> /// OneBot 12 标准事件
> pub type Event = BaseEvent<EventContent>;
> pub type MessageEvent = BaseEvent<MessageContent<MessageEventDetail>>;
> pub type NoticeEvent = BaseEvent<NoticeContent>;
> pub type RequestEvent = BaseEvent<RequestContent>;
> pub type MetaEvent = BaseEvent<MetaContent>;
> ```

当需要扩展事件时（扩展 detail_type ）, 请使用 untagged enum 来实现：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedMeta {
    Standard(MetaContent),
    Extended(<YourMetaContent>),
}

// or

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedEventContent {
    Standard(EventContent),
    Extended(<YourEventContent>),
}
```

当然，你也可以自由定义 Content 枚举，甚至 BaseEvent ( 并不建议怎么做 )。