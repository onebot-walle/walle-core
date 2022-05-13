# 0.3.1

- 修复心跳包无 type 字段 bug

# 0.3.0

- 修复 `tokio-tungstenite 0.17` 默认不再为 request 添加 headers 的问题
- Hanlder 变更为一个泛型传入实例。