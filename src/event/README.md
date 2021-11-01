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
    Meta(Meta),
    Message(Message),
    Notice(Notice),
    Request(Request),
}
```

EventContent 使用 type 字段区分不同的类型，因此符合 Onebot 的标准事件可以反序列化为 `BaseEvent<EventContent>` 类型。

> 该类型可以直接使用类型别名（alias）: Event
> ```rust
> /// OneBot 12 标准事件
> pub type Event = BaseEvent<EventContent>;
> ```

当需要扩展事件时（扩展 detail_type ）, 可以使用 `ExtendedContent`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ExtendedContent<M, E, N, R> {
    Meta(ExtendedMeta<M>),
    Message(ExtendedMessage<E>),
    Notice(ExtendedNotice<N>),
    Request(ExtendedRequest<R>),
}
```

其中每种扩展均为以下类似类型：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedMeta<T> {
    Standard(Meta),
    Extended(T),
}
```

这是一个 untagged 枚举，serde 将会尝试所有可能匹配， 这也意味着在你的扩展 content 类型 T 中，你依然可以使用 detail_type 字段进行反序列化操作。

当然，你也可以自由定义 Content 枚举( 并不建议怎么做 )，但是你的 Content 需要实现 Trait EventContentExt

```rust
pub trait EventContentExt {
    fn from_standard(content: EventContent) -> Self;
}
```

由于某些原因（建立心跳包），lib 开发者需要这个 Trait 。