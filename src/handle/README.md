# Handlers

Handler 是 OneBot 网络通讯与你的实现或应用的接口

> 下文提到的 Event、Action、ActionResp 需要在定义 Handler Trait 时指定具体类型

对于实现端，收到 Action 时，将会调用 ActionHandler

对于应用端，收到 Evnet 时，将会调用 EventHandler ；收到 ActionResp 将会调用 ActionRespHandler