# Walle-core

[![OneBot](https://img.shields.io/badge/OneBot-12-black?logo=data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAHAAAABwCAMAAADxPgR5AAAAGXRFWHRTb2Z0d2FyZQBBZG9iZSBJbWFnZVJlYWR5ccllPAAAAAxQTFRF////29vbr6+vAAAAk1hCcwAAAAR0Uk5T////AEAqqfQAAAKcSURBVHja7NrbctswDATQXfD//zlpO7FlmwAWIOnOtNaTM5JwDMa8E+PNFz7g3waJ24fviyDPgfhz8fHP39cBcBL9KoJbQUxjA2iYqHL3FAnvzhL4GtVNUcoSZe6eSHizBcK5LL7dBr2AUZlev1ARRHCljzRALIEog6H3U6bCIyqIZdAT0eBuJYaGiJaHSjmkYIZd+qSGWAQnIaz2OArVnX6vrItQvbhZJtVGB5qX9wKqCMkb9W7aexfCO/rwQRBzsDIsYx4AOz0nhAtWu7bqkEQBO0Pr+Ftjt5fFCUEbm0Sbgdu8WSgJ5NgH2iu46R/o1UcBXJsFusWF/QUaz3RwJMEgngfaGGdSxJkE/Yg4lOBryBiMwvAhZrVMUUvwqU7F05b5WLaUIN4M4hRocQQRnEedgsn7TZB3UCpRrIJwQfqvGwsg18EnI2uSVNC8t+0QmMXogvbPg/xk+Mnw/6kW/rraUlvqgmFreAA09xW5t0AFlHrQZ3CsgvZm0FbHNKyBmheBKIF2cCA8A600aHPmFtRB1XvMsJAiza7LpPog0UJwccKdzw8rdf8MyN2ePYF896LC5hTzdZqxb6VNXInaupARLDNBWgI8spq4T0Qb5H4vWfPmHo8OyB1ito+AysNNz0oglj1U955sjUN9d41LnrX2D/u7eRwxyOaOpfyevCWbTgDEoilsOnu7zsKhjRCsnD/QzhdkYLBLXjiK4f3UWmcx2M7PO21CKVTH84638NTplt6JIQH0ZwCNuiWAfvuLhdrcOYPVO9eW3A67l7hZtgaY9GZo9AFc6cryjoeFBIWeU+npnk/nLE0OxCHL1eQsc1IciehjpJv5mqCsjeopaH6r15/MrxNnVhu7tmcslay2gO2Z1QfcfX0JMACG41/u0RrI9QAAAABJRU5ErkJggg==)](https://github.com/botuniverse/onebot/pull/108)
<a href="https://github.com/abrahum/Walle-core/blob/master/license">
  <img src="https://img.shields.io/github/license/abrahum/Walle-core" alt="license">
</a>
<a href="https://crates.io/crates/walle-core">
  <img src="https://img.shields.io/crates/v/walle-core">
</a>


Walle-core 是一个 Rust OneBot Lib ( 不同于 libonebot 他同样可以应用于 OneBot 应用端 )

Walle 的名字来源于机械总动员的 WALL-E ( A Rusty Bot )

## 功能

- 提供 OneBot v12 标准 Event、Action、ActionResp 序列化与反序列化功能，并支持自定义扩展
- 提供 OneBot v12 实现端标准网络通讯协议
- 提供 OneBot v12 应用端标准网络通讯协议（Http HttpWebhook 未支持）

## features

- ~~echo~~: 启用 echo 字段 ( echo 字段默认实现，才不是因为分离太难了 )
- http: 启用 Http 与 HttpWebhook 通讯协议
- websocket: 启用正向 WebSocket 与反向 WebSocket 通讯协议
- impl: 启用实现端 lib api
- app: 启用应用端 lib api

## How to use

仅展示最小实例

### Implementation

```rust
use walle_core::{ImplConfig, impls::OneBot, DefaultHandler};

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace"); 
    tracing_subscriber::fmt().with_env_filter(env).init(); // 初始化 tracing
    let config = ImplConfig::default();
    let ob = OneBot::new(
        "Your impl name".to_owned(),
        "Your impl platform".to_owned(),
        "Your bot self id".to_owned(),
        config,
        DefaultHandler::arc(), // ActionHandler
        ).arc();
    ob.run().await
}
```

### Application

```rust
use walle_core::{AppConfig, app::OneBot, DefaultHandler};

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace"); 
    tracing_subscriber::fmt().with_env_filter(env).init(); // 初始化 tracing
    let config = AppConfig::default();
    let ob = OneBot::new(
        config, 
        DefaultHandler::arc(), // EventHandler
    ).arc();
    ob.run().await
}
```