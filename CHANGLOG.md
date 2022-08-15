# 1.0.0(a1)

- refactor handler trait
- support OneBot rfc #180 #181 #204

# 0.6.1

- fix Impl-OBC self_id won't update error

# 0.6.0

- remove action.kick_group_member
- remove action.ban_group_member
- remove action.unban_group_member
- remove action.set_group_admin
- remove action.unset_group_admin
- remove notice.group_member_ban
- remvoe notice.group_member_unban
- remove notice.group_admin_set
- remove notice.group_admin_unset
- new OneBot and all models

# 0.5.4

- http 反序列化错误（serde_json from_reader 潜在bug？）

# 0.5.3

- bug fix

# 0.5.2

- impl IntoMessage for MessageSegment
- fix MessageSegment::Reply Serialize bug

# 0.5.1

- bug fix

# 0.5.0

- 添加 EventType trait
- 修复了一些 feature 相关的 bug
- app 端启用 Http Webhook 和 MsgPack 格式功能

# 0.4.0 

- MessageContent extra 扩展字段变更为由 MessageEventDetail 持有
- MessageContent 增加 D 泛型，可支持更多 DetailType 模型
- RespContent 增加 E 泛型，为支持扩展 Event 模型
- ExtendedValue 添加 `Bytes(Vec<u8>)` 枚举类型
- 移除部分无用 Error
- Bot Api 变更为 BotActionExt trait 重新重构实现
- 添加 RespError 和 RespStatusExt trait
- rfc #154 添加两级群组支持
- 添加 extended macros

# 0.3.1

- 修复心跳包无 type 字段 bug

# 0.3.0

- 修复 `tokio-tungstenite 0.17` 默认不再为 request 添加 headers 的问题
- Hanlder 变更为一个泛型传入实例。