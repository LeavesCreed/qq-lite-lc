# qq-lite-lc

一个以 Rust 为核心、使用 NapCat 作为消息后端的轻量 QQ 客户端实验项目。

## Architecture

当前仓库按 workspace 组织，核心目标是让 GUI 和 TUI 共享同一套客户端能力：

```text
crates/
  qq-core        # 领域模型、命令/事件、状态流、view model
  qq-napcat      # NapCat / OneBot 11 DTO、WebSocket adapter、协议映射
  qq-render      # RichNode 到 GUI/TUI 输出的渲染适配
  qq-store       # SQLite 持久化
  qq-tui         # Ratatui 终端客户端
apps/
  desktop        # Tauri v2 + Svelte 桌面客户端
```

核心数据流：

```text
NapCat WebSocket -> OneBot DTO -> DomainEvent -> ClientCore -> ViewModel -> GUI/TUI
GUI/TUI Action -> ClientCommand -> NapCat adapter
```

首版只实现极简文本链路：

- 文本消息收发
- 会话列表基础模型
- 消息时间线基础模型
- 非文本消息降级为 unsupported 占位
- SQLite 缓存会话和消息

## Requirements

- Rust toolchain with Cargo
- Node.js and npm
- A running NapCat instance with WebSocket Server enabled

## Desktop GUI

```powershell
cd apps/desktop
npm install
npm run tauri dev
```

默认连接地址可在界面中填写，例如：

```text
ws://127.0.0.1:3001
```

## TUI

```powershell
$env:NAPCAT_WS = "ws://127.0.0.1:3001"
cargo run -p qq-tui
```

如果 NapCat 配置了 access token：

```powershell
$env:NAPCAT_TOKEN = "your-token"
```

## Current Status

这是第一版架构落地，重点是边界和数据流：

- UI 不直接依赖 NapCat DTO
- NapCat adapter 只负责协议连接和转换
- `qq-core` 持有领域状态和 view model
- GUI/TUI 通过同一套 command/event/view model 工作

后续重点：

- 将发送消息的 optimistic pending 状态写入 core/store
- 扩展 RichNode：图片、表情、@、回复、链接
- 改进 NapCat 长连接上的 action response / echo 匹配
- 为 core、mapper、store 增加单元测试和集成测试
